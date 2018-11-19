window.onload = () => {
    login_button = document.getElementById("login-button");
    logout_button = document.getElementById("logout-button");
    login_input = document.getElementById("login-input");
    users_div = document.getElementById("users");
    message_input = document.getElementById("message-input");
    messages_div = document.getElementById("messages");
    send_button = document.getElementById("send-button");

    toAuthorized(false);

    login_button.onclick = () => {
        var a = new XMLHttpRequest();
        a.open("POST", "http://127.0.0.1:1337/login", true);
        a.onload = function (e) {
            if (a.responseText === "")
                return;
            resp = JSON.parse(a.responseText);
            token = resp.token;
            toAuthorized(true);
            runUsersFetchLoop(token);
            runMessagesFetchLoop(token);
        }
        a.send(JSON.stringify({ username: login_input.value }));
    }

    logout_button.onclick = () => {
        var a = new XMLHttpRequest();
        a.open("POST", "http://127.0.0.1:1337/logout", true);
        a.setRequestHeader('Authorization', `Token ${token}`)
        a.onload = function (e) {
            if (a.responseText === "")
                return;
            token = null;
            toAuthorized(false);
        }
        a.send();
    }

    send_button.onclick = () => {
        var a = new XMLHttpRequest();
        a.open("POST", "http://127.0.0.1:1337/messages", true);
        a.setRequestHeader('Authorization', `Token ${token}`)
        a.onload = function (e) {
            if (a.responseText === "")
                return;
        }
        a.send(JSON.stringify({"message": message_input.value }));
    }

    function runUsersFetchLoop(token) {
        var a = new XMLHttpRequest();
        a.open("GET", "http://127.0.0.1:1337/users", true);
        a.setRequestHeader('Authorization', `Token ${token}`)
        a.onload = function (e) {
            if (a.responseText === "")
                return;

            updateUsers(JSON.parse(a.responseText));
            setTimeout(() => runUsersFetchLoop(token), 1000);
        }
        a.send();
    }

    function runMessagesFetchLoop(token) {
        var a = new XMLHttpRequest();
        a.open("GET", `http://127.0.0.1:1337/messages?offset=${last_message_index}&count=${last_message_index + 10}`, true);
        a.setRequestHeader('Authorization', `Token ${token}`)
        a.onload = function (e) {
            if (a.responseText === "")
                return;

            updateMessages(JSON.parse(a.responseText));
            setTimeout(() => runMessagesFetchLoop(token), 1000);
        }
        a.send();
    }

    function updateUsers(usersJson) {
        usersJson.users.forEach(user => {
            id = user.id;
            if (users[id] === undefined) {
                div = users_div.appendChild(document.createElement('div'));
                div.class = "user";
                users[id] = { div: div };
            }
            users[id].div.innerText = `${user.id} ${user.username} ${user.online}`;
        });
    }

    function updateMessages(messagesJson) {
        messagesJson.messages.forEach(message => {
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

    function toAuthorized(isAuthorized) {
        if (!isAuthorized) {
            token = null;
            last_message_index = 0;
        }
        function disabled(isAuthorized) { return isAuthorized ? null : "disabled" };
        login_button.disabled = !disabled(isAuthorized);
        login_input.disabled = !disabled(isAuthorized);
        logout_button.disabled = disabled(isAuthorized);
        users_div.hidden = !isAuthorized;
        message_input.hidden = !isAuthorized;
        send_button.hidden = !isAuthorized;
        messages_div.hidden = !isAuthorized;
    }
}