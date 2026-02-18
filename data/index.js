let socket =new WebSocket("http://localhost:3000");

console.log(socket);

socket.onmessage = (env) =>{
    const data = JSON.parse(env.data)
    console.log(data);
}