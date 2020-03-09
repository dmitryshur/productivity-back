FROM postgres:12.2

RUN apt-get update \
    && apt-get install curl lsb-release -y \
    && curl -s https://packagecloud.io/install/repositories/golang-migrate/migrate/script.deb.sh | bash \
    && apt-get update \
    && apt-get install migrate=4.9.1 -y

