window.onload = () => {
    let login_button = document.getElementById("login-button");
    let logout_button = document.getElementById("logout-button");
    let username_input = document.getElementById("login-input");
    let reconnect_button = document.getElementById("reconnect-button");
    let users_div = document.getElementById("users");
    let message_input = document.getElementById("message-input");
    let messages_div = document.getElementById("messages");
    let send_button = document.getElementById("send-button");

    socket = null;
    token = "";
    to_authorized(false);
    connect();

    reconnect_button.onclick = (ev) => {
        connect()
    }

    function connect() {
        socket = new WebSocket("ws://127.0.0.1:1337");
        socket.onmessage = (ev) => {
            let resp = JSON.parse(ev.data);
            let method = resp.method;
            if (method == 'LoginResult') {
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
            if (method == "Users") {
                update_users(JSON.parse(resp.data).users)
            }
        };
        socket.onclose = (ev) => {
            to_authorized(false);
            reconnect_button.hidden = false;
        }

        socket.onopen = (ev) => {
            if (token !== "") {
                socket.send(JSON.stringify({
                    method: "TokenLogin",
                    data: token
                }))
            }
    
            reconnect_button.hidden = true;
        }
    }

    login_button.onclick = () => {
        socket.send(JSON.stringify({
            method: "NameLogin",
            data: username_input.value
        }))
    }

    logout_button.onclick = () => {
        socket.send(JSON.stringify({
            method: "Logout",
            data: username_input.value
        }))
        to_authorized(false);
    }

    send_button.onclick = () => {
        socket.send(JSON.stringify({
            method: "Messages",
            data: message_input.value
        }))
    }

    function to_authorized(is_authorized) {
        if (!is_authorized) {
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
        users_json.forEach(user => {
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