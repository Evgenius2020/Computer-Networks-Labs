window.onload = () => {
    let login_button = document.getElementById("login-button");
    let logout_button = document.getElementById("logout-button");
    let username_input = document.getElementById("login-input");
    let users_div = document.getElementById("users");
    let message_input = document.getElementById("message-input");
    let messages_div = document.getElementById("messages");
    let send_button = document.getElementById("send-button");

    to_authorized(false);

    let socket = new WebSocket("ws://192.168.0.104:1337")
    socket.onmessage = (ev) => {
        let resp = JSON.parse(ev.data);
        let method = resp.method;
        if (method == 'Login') {
            let login_result = JSON.parse(resp.data);
            if (login_result === null) {
                return;
            }

            to_authorized(true);
            token = login_result.token;
        }
        if (method == "Messages") {
            update_messages(JSON.parse(resp.data).messages)
        }
    }

    login_button.onclick = () => {
        socket.send(JSON.stringify({
            method: "Login",
            data: username_input.value
        }))
    }

    logout_button.onclick = () => {
        socket.send(JSON.stringify({
            method: "Logout",
            data: username_input.value
        }))
    }

    send_button.onclick = () => {
        socket.send(JSON.stringify({
            method: "Messages",
            data: message_input.value
        }))
    }

    function to_authorized(is_authorized) {
        if (!is_authorized) {
            token = null;
            last_message_index = 0;
        }
        function disabled(isAuthorized) { return isAuthorized ? null : "disabled" };
        login_button.disabled = !disabled(is_authorized);
        username_input.disabled = !disabled(is_authorized);
        logout_button.disabled = disabled(is_authorized);
        users_div.hidden = !is_authorized;
        message_input.hidden = !is_authorized;
        send_button.hidden = !is_authorized;
        messages_div.hidden = !is_authorized;
    }

    function update_users(users_json) {
        users_json.users.forEach(user => {
            id = user.id;
            if (users[id] === undefined) {
                div = users_div.appendChild(document.createElement('div'));
                div.class = "user";
                users[id] = { div: div };
            }
            users[id].div.innerText = `${user.id} ${user.username} ${user.online}`;
        });
    }

    function update_messages(messages_json) {
        console.log(messages_json)
        messages_json.forEach(message => {
            id = message.id;
            if (id > last_message_index)
                last_message_index = id;
            if (messages[id] === undefined) {
                div = messages_div.appendChild(document.createElement('div'));
                div.class = "message";
                messages[id] = { div: div };
            }
            messages[id].div.innerText = `${message.id} ${message.message} ${message.author}`;
        });
    }
}