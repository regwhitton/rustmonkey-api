# docker build -t local_lambci:provided -f debugging-lambda-dockerfile
FROM lambci/lambda:provided

USER root
RUN yum install -y lldb
RUN yum install -y gdb

