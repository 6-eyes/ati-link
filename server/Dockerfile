FROM alpine:latest

COPY target/debug/client /data/
CMD chmod +x ./data/client
ENTRYPOINT ["/data/client"]
