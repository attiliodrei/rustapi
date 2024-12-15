# rustapi
rustapi


curl http://localhost:8080/users
curl     http://localhost:8080/users/1
curl     http://localhost:8080/users/2
curl -X POST http://localhost:8080/users \  
     -H "Content-Type: application/json" \
     -d '{
         "username": "drei",
         "email": "drei@example.com"
     }'

Delete a User
curl -X DELETE http://localhost:8080/users/1
