FROM alpine:latest

COPY target/debug/server /data/
CMD chmod +x ./data/server
ENTRYPOINT ["/data/server"]
