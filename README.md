# todo-api
todo-api is a Rust library that stores todo in postgres database with axum and sqlx.

## Installation
Use the package manager cargo or docker to install todo-api.

## Usage
Postgres table todo will be created automatically because migrations are added. 

check service is alive:
```bash
curl "http://localhost:8080/alive"
```

check service is ready:
```bash
curl "http://localhost:8080/ready"
```

get all todos from database:
```bash
curl --request GET --url "http://localhost:8080/todos"
```

create todo:
```bash
curl --request POST --url "http://localhost:8080/todos" --header 'Content-Type: application/json' --data '{ "body": "foo" }'
```

read created todo by id:
```bash
curl --request GET --url "http://localhost:8080/todos/1"
```

update created todo by id:
```bash
curl --request PUT --url "http://localhost:8080/todos/1" --header 'Content-Type: application/json' --data '{ "body": "foo", "completed": true }'
```

delete created todo by id:
```bash
curl --request DELETE --url http://localhost:8080/todos/1
```
