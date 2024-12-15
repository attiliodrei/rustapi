# rustapi
rustapi

### Get all User
```bash
curl http://localhost:8080/users
```
### Get all User by id

```bash
curl     http://localhost:8080/users/1
```

### add a User
```bash
curl -X POST http://localhost:8080/users \
     -H "Content-Type: application/json" \
     -d '{
         "username": "drei",
         "email": "drei@example.com"
     }'
```
### Delete a User
```bash
curl -X DELETE http://localhost:8080/users/1
```



