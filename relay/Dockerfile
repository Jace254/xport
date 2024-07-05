FROM crystallang/crystal:latest

WORKDIR /app

COPY shard.yml shard.lock ./
RUN shards install --production

COPY . .
RUN crystal build --release src/relay.cr

EXPOSE 443
CMD ["./relay"]