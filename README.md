# Проект Blog
Реализованы приложения: 
- сервер (http, grpc)
- cli утилита (http, grpc)
- фронтенд (http)
Также реализованы 2 библиотеки:
- blog-client - вспомогательная библиотека с клиентскими запросами и типами
- blog-wasm - генератор wasm модуля для клиентских запросов

Для старта сервера нужно запустить postgres, выполнить сборку и запустить сервер. 
Предварительно нужно создать .env файл, пример содержимого: 
```dotenv
DATABASE_URL=postgres://blog:blogpass@localhost/postgres
JWT_SECRET=jasidjasdasjduiasdiasudjasuidjasdiuasndiuasdjasdjas8djad
HTTP_PORT=3000
GRPC_PORT=50051
```
Для сборки и запуска нужно выполнить
```shell
cargo build
./target/debug/blog-server
```
Для запуска фронтенда нужно сгенерировать wasm модуль и запустить фронтенд
```shell
cd blog-wasm
wasm-pack build --target web
python3 -m http.server 8000
```
Также можно выполнять запросы через cli команды
```shell
./target/debug/blog-cli --server http://localhost:3000 register --username "ivan" --email "ivan@example.com" --password "secret123"
./target/debug/blog-cli --server http://localhost:50051 --grpc register --username "anton" --email "anton@example.com" --password "secret123"
```

## Примечания
### BlogClient
Некоторые решения были реализованы по описанию задания, это местами приводит к дублированию кода или избыточным абстракциям (выбран путь дублирования).
Например, держать в BlogClient вместе сущности gprc client, http client и transport (кажется избыточным).