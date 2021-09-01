FROM ubuntu

RUN apt-get update && apt-get install -y nmap  && rm -rf /var/lib/apt/lists/*

RUN mkdir data

RUN mkdir src

EXPOSE 7000
EXPOSE 8000

CMD ["./src/mapreduce"]