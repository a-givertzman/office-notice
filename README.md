## Run in Docker container

To run the bot in container:
- Place a configuration file `config.yaml` of the following format in project root:
```yaml
bot:
  connection:
    name: ${BOT_NAME}
    token: "${TELEGRAM_BOT_TOKEN}"
```
- Run command from terminal in project root:
```bash
docker compose -f ./docker/docker-compose.yaml up -d
```
- Check that container STATUS is `Up`, using command from terminal:
```bash
docker ps
```