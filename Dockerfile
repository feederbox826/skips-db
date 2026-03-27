FROM scratch
ARG TARGETARCH

COPY --chmod=555 docker/${TARGETARCH}/skips-db /skips-db

ENTRYPOINT ["/skips-db"]