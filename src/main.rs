use std::collections::HashMap;
use std::time::{Duration, Instant};

use actix::*;
use actix_files as fs;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

use serde_json::{Result as JsonResult, Value};

mod server;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Entry point for our route
async fn web_socket_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::WebSocketServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        WsWebSocketSession {
            id: 0,
            hb: Instant::now(),
            room: "Lobby".to_string(),
            name: "TEMPORARY NAME?".to_string(), // TODO
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

struct WsWebSocketSession {
    /// unique session id
    id: usize,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
    /// joined room
    room: String,
    /// peer name
    name: String,
    /// web socket server
    addr: Addr<server::WebSocketServer>,
}

impl Actor for WsWebSocketSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // start heart beating
        self.hb(ctx);

        // register self in web socket server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsWebSocketSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        self.addr
            .send(server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with web socket server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify web socket server
        self.addr.do_send(server::Disconnect { id: self.id });
        Running::Stop
    }
}

/// Handle messages from web socket server, we simply send it to peer websocket
impl Handler<server::Message> for WsWebSocketSession {
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsWebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        // println!("WEBSOCKET MESSAGE: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let m = text.trim();
                // we check for /sss type of messages

                let testmsg: JsonResult<HashMap<String, Value>> = serde_json::from_str(m);
                match testmsg {
                    Err(_) => println!("Malformatted messge detected: {}", text),
                    Ok(jsonmsg) => {
                        println!("{:?}", jsonmsg);

                        let r#type = match jsonmsg["type"].as_str() {
                            Some(res) => res,
                            None => "NOT PARSEABLE",
                        };

                        match r#type {
                            "join" => {
                                let room = jsonmsg["room"].as_str();
                                let name = jsonmsg["name"].as_str();

                                if room.is_some() && name.is_some() {
                                    self.addr.do_send(server::Join {
                                        user_id: self.id,
                                        user_name: name.unwrap().to_string(),
                                        room_name: room.unwrap().to_string(),
                                    });
                                    self.name = name.unwrap().to_string();
                                    self.room = room.unwrap().to_string();
                                }
                            }
                            "raise" => match jsonmsg["raiseobject"].as_str() {
                                Some(object) => self.addr.do_send(server::Raise {
                                    object: object.to_string(),
                                    owner_id: self.id,
                                    owner_name: self.name.clone(),
                                    room_name: self.room.to_owned(),
                                }),
                                None => (),
                            },
                            "lower" => match jsonmsg["lowerobject"].as_str() {
                                Some(object) => self.addr.do_send(server::Lower {
                                    object: object.to_string(),
                                    owner_id: self.id,
                                    owner_name: self.name.clone(),
                                    room_name: self.room.to_owned(),
                                }),
                                None => (),
                            },
                            _ => (),
                        }
                    }
                };
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}

impl WsWebSocketSession {
    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // notify web socket server
                act.addr.do_send(server::Disconnect { id: act.id });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Start web socket server actor
    let server = server::WebSocketServer::default().start();

    // Create Http server with websocket support
    HttpServer::new(move || {
        App::new()
            .data(server.clone())
            // redirect to websocket.html
            .service(web::resource("/").route(web::get().to(|| {
                HttpResponse::Found()
                    .header("LOCATION", "/static/websocket.html")
                    .finish()
            })))
            // websocket
            .service(web::resource("/ws/").to(web_socket_route))
            // static resources
            .service(fs::Files::new("/static/", "static/"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
