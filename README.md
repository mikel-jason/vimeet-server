# **Vi**sual **Mee**ting **T**ool

University project by [JulianGroshaupt](https://www.github.com/JulianGroshaupt) and [sarcaustech](https://www.github.com/sarcaustech)

# Using Docker
The containerization is set up with [multi-staging](https://docs.docker.com/develop/develop-images/multistage-build/) to build an image without the compilation overhead of Rust / Cargo (see [Dockerfile](Dockerfile)).

To build the docker image, run the following in the project's directory:

```docker build -t vimeet-server .```

To create and start a container, run:

```docker run -itd -p <YOUR_HOST_PORT>:8080 --name <YOUR_CONTAINER_NAME> vimeet-server```

# References
* [Official Rust image on Docker Hub](https://hub.docker.com/_/rust)
