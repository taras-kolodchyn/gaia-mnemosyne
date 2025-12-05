FROM python:3.11-slim

WORKDIR /app

RUN pip install --no-cache-dir tensorzero

COPY tensorzero.yaml /app/tensorzero.yaml

EXPOSE 9090

ENTRYPOINT ["tensorzero", "serve", "--config", "/app/tensorzero.yaml"]
