INSERT INTO user (id, username, name) VALUES 
(1, "john", "John Doe"),
(2, "jane", "Jane Doe"),
(3, "fred", "Fred Nerk");

INSERT INTO conversation (id, sender_id, receiver_id, last_message_id) VALUES
(1, 1, 2, NULL),
(2, 1, 3, NULL);

INSERT INTO message (id, content, sender_id, conversation_id, previous_message_id) VALUES
(1, "Hi Jane! I would like to buy a few oranges, are they fresh?", 1, 1, NULL),
(2, "Yes John! I just collected them this morning!", 2, 1, 1),
(3, "Thank you for the clarification!", 1, 1, 2),
(4, "Hi John! Is your orange cake made from fresh oranges?", 3, 2, NULL),
(5, "Yes Fred! I bought them today from Jane!", 1, 2, 4),
(6, "Amazing! That makes me relieved, thank you!", 3, 2, 5);
