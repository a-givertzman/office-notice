# Office notice

## 1. Access roles


| Role<br>name | Request<br>access | Subscribe,<br>Read | Send<br>Notice | Grant<br>Acces | Full<br>Access |
| --        | :--:           | :--:      | :--:        | :--:        | :--:        |
| Guest | ✔ | - | - | - | - |
| Member | ✔ | ✔ | - | - | - |
| Sender | ✔ | ✔ | ✔ | - | - |
| Moder | ✔ | ✔ | ✔ | ✔ | - |
| Admin | ✔ | ✔ | ✔ | ✔ | ✔ |

### For manual granting user access level:

- User send /start to the bot
- Admin open the file `assets/users.json`
- Find user by UserName
  - Add role to user
  ```json
    "0000000000": {
      "id": "0000000000",
      "name": "UserName",
      "contact": "-",
      "address": null,
      "subscriptions": [],
      "last_seen": "2024-10-25T14:40:51.891852847+00:00",
      "role": [
        - "Guest"
        + "NewRole"
      ]
    }

  ```

## 2. Run in Docker container

To run the bot in container:
- Place a configuration file `config.yaml` of the following format in project root:
```yaml
bot:
  connection:
    name: ${BOT_NAME}
    token: "${TELEGRAM_BOT_TOKEN}"
```
- (Optional) If you want to build behind network proxy, add corresponding build args to `./docker/docker-compose.yaml` file as shown below:
```yaml
services:
  office-notice-telegram-bot:
    build:
      #...
      args:
        HTTP_PROXY: http://${USER_NAME}:${PASSWORD}@${IP}:${PORT}/
        HTTPS_PROXY: http://${USER_NAME}:${PASSWORD}@${IP}:${PORT}/
```
- (Optional) If you want to run behind network proxy, add corresponding environment values to `./docker/docker-compose.yaml` file as shown below:
```yaml
services:
  office-notice-telegram-bot:
    #...
    environment:
      http_proxy: http://${USER_NAME}:${PASSWORD}@${IP}:${PORT}/
      https_proxy: http://${USER_NAME}:${PASSWORD}@${IP}:${PORT}/
    #...
```
- Run command from terminal in project root:
```bash
docker compose -f ./docker/docker-compose.yaml up -d
```
- Check that container STATUS is `Up`, using command from terminal:
```bash
docker ps
```