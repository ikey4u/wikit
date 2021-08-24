FROM archlinux:base-devel
MAINTAINER ikey4u "pwnkeeper@gmail.com"
RUN mkdir -p /wikitdev
COPY /Dockerfile.sh /wikitdev/
RUN bash --login /wikitdev/Dockerfile.sh add_utilities
RUN bash --login /wikitdev/Dockerfile.sh add_win_toolchain
RUN bash --login /wikitdev/Dockerfile.sh add_mac_toolchain