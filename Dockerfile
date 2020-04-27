FROM ekidd/rust-musl-builder:stable AS builder

WORKDIR /usr/vimeet-server
RUN sudo chown -R rust:rust .
COPY . .
RUN cargo build --release


FROM alpine:3.11 
LABEL Maintainer "Mikel Muennekhoff <inf18207@lehre.dhbw-stuttgart.de>"

WORKDIR /usr/vimeet-server

COPY --from=builder /usr/vimeet-server/target/x86_64-unknown-linux-musl/release/vimeet-server .
COPY --from=builder /usr/vimeet-server/static /usr/vimeet-server/static 
COPY --from=builder /usr/vimeet-server/.env /usr/vimeet-server/.env 

# Overruling .env 
ENV PORT 8080
RUN sed -ie "s/VIMEET_BIND_ADDRESS=.*//g" .env
RUN echo "VIMEET_BIND_ADDRESS=0.0.0.0" >> .env

CMD [ "./vimeet-server" ]


EXPOSE 8080
