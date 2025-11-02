CREATE TABLE IF NOT EXISTS user (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE,
    name TEXT
);

CREATE TABLE IF NOT EXISTS conversation (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sender_id INTEGER NOT NULL,
    receiver_id INTEGER NOT NULL,
    last_message_id INTEGER,
    unread_for_sender INTEGER,
    unread_for_receiver INTEGER,
    FOREIGN KEY(sender_id) REFERENCES user(id),
    FOREIGN KEY(receiver_id) REFERENCES user(id),
    FOREIGN KEY(last_message_id) REFERENCES message(id)
);

CREATE TABLE IF NOT EXISTS message (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sender_id INTEGER NOT NULL,
    conversation_id INTEGER NOT NULL,
    content BLOB,
    previous_message_id INTEGER,
    FOREIGN KEY(sender_id) REFERENCES user(id),
    FOREIGN KEY(conversation_id) REFERENCES conversation(id),
    FOREIGN KEY(previous_message_id) REFERENCES message(id)
);
