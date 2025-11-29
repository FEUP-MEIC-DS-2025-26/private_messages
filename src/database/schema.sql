CREATE TABLE IF NOT EXISTS user (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE,
    name TEXT
);

CREATE TABLE IF NOT EXISTS conversation (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id INTEGER NOT NULL,
    seller_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    last_message_id INTEGER,
    unread_for_sender INTEGER,
    unread_for_receiver INTEGER,
    FOREIGN KEY(client_id) REFERENCES user(id),
    FOREIGN KEY(seller_id) REFERENCES user(id),
    FOREIGN KEY(product_id) REFERENCES product(id),
    FOREIGN KEY(last_message_id) REFERENCES message(id)
);

CREATE TABLE IF NOT EXISTS message (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sender_id INTEGER NOT NULL,
    conversation_id INTEGER NOT NULL,
    content BLOB,
    salt BLOB NOT NULL,
    timestamp DATETIME NOT NULL,
    previous_message_id INTEGER,
    FOREIGN KEY(sender_id) REFERENCES user(id),
    FOREIGN KEY(conversation_id) REFERENCES conversation(id),
    FOREIGN KEY(previous_message_id) REFERENCES message(id)
);

CREATE TABLE IF NOT EXISTS product (
    id INTEGER PRIMARY KEY,
    seller_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    FOREIGN KEY(seller_id) REFERENCES user(id)
);
