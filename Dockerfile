FROM archlinux:base-devel
MAINTAINER ikey4u "pwnkeeper@gmail.com"
RUN mkdir -p /wikitdev
COPY /scripts/Dockerfile.sh /wikitdev/
RUN bash /wikitdev/Dockerfile.sh