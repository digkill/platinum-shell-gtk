use chrono::Utc;
use rusqlite::{params, Connection};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const SCHEMA_VERSION: i32 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactRecord {
    pub name: String,
    pub phone: String,
    pub note: String,
    pub created_at: i64,
    pub last_called_at: i64,
    pub last_message_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadKind {
    Direct,
    Group,
    Relay,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageRecord {
    pub author: String,
    pub body: String,
    pub outgoing: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageThread {
    pub title: String,
    pub status: String,
    pub preview: String,
    pub messages: Vec<MessageRecord>,
    pub created_at: i64,
    pub last_message_at: i64,
    pub kind: ThreadKind,
    pub updated_at: i64,
}

#[derive(Debug, Clone)]
pub struct LauncherStore {
    contacts: Vec<ContactRecord>,
    db_path: PathBuf,
    threads: Vec<MessageThread>,
}

impl LauncherStore {
    pub fn load_or_init(network_online: bool) -> Self {
        let db_path = store_db_path();
        let _ = init_parent_dir(&db_path);

        if let Ok(Some(store)) = Self::load_from_db(&db_path) {
            return store;
        }

        let seeded = Self::seeded(network_online, db_path);
        let _ = seeded.persist();
        seeded
    }

    fn seeded(network_online: bool, db_path: PathBuf) -> Self {
        let now = now_ts();
        let contacts = vec![
            contact("Operator Desk", "+7 912 440 77 90", "Primary", now - 240),
            contact("Base Station", "+7 912 440 12 01", "Pinned", now - 180),
            contact("Field Team", "+7 912 440 53 20", "Shared", now - 120),
            contact("Emergency Link", "112", "Priority", now - 60),
        ];

        let threads = vec![
            MessageThread {
                title: String::from("Field Team"),
                status: String::from("3 unread"),
                preview: String::from("Rendezvous updated for 22:30, confirm route lock."),
                messages: vec![
                    message(
                        "Field Team",
                        "Rendezvous updated for 22:30.",
                        false,
                        now - 300,
                    ),
                    message(
                        "You",
                        "Received. Syncing route and battery window.",
                        true,
                        now - 240,
                    ),
                    message(
                        "Field Team",
                        "Confirm route lock when ready.",
                        false,
                        now - 180,
                    ),
                ],
                created_at: now - 300,
                last_message_at: now - 180,
                kind: ThreadKind::Group,
                updated_at: now - 180,
            },
            MessageThread {
                title: String::from("Contacts Queue"),
                status: String::from("Updated"),
                preview: String::from("Three follow-ups extracted from recent assistant activity."),
                messages: vec![
                    message(
                        "Contacts Queue",
                        "Three follow-ups extracted from assistant context.",
                        false,
                        now - 160,
                    ),
                    message("You", "Queue them for morning review.", true, now - 130),
                ],
                created_at: now - 160,
                last_message_at: now - 130,
                kind: ThreadKind::System,
                updated_at: now - 130,
            },
            MessageThread {
                title: String::from("Relay Bridge"),
                status: String::from(if network_online { "Linked" } else { "Standby" }),
                preview: String::from("Relay tunnel prepared for device-to-device handoff."),
                messages: vec![
                    message(
                        "Relay Bridge",
                        "Relay tunnel prepared for device handoff.",
                        false,
                        now - 110,
                    ),
                    message(
                        "Relay Bridge",
                        if network_online {
                            "Transport available. Ready for outbound sync."
                        } else {
                            "Transport unavailable. Waiting for network."
                        },
                        false,
                        now - 90,
                    ),
                ],
                created_at: now - 110,
                last_message_at: now - 90,
                kind: ThreadKind::Relay,
                updated_at: now - 90,
            },
        ];

        Self {
            contacts,
            db_path,
            threads,
        }
    }

    pub fn contacts_snapshot(&self) -> Vec<ContactRecord> {
        let mut contacts = self.contacts.clone();
        contacts.sort_by(|left, right| left.name.cmp(&right.name));
        contacts
    }

    pub fn recents_snapshot(&self) -> Vec<ContactRecord> {
        let mut contacts = self.contacts.clone();
        contacts.sort_by(|left, right| {
            right
                .last_called_at
                .cmp(&left.last_called_at)
                .then_with(|| right.last_message_at.cmp(&left.last_message_at))
                .then_with(|| right.updated_at.cmp(&left.updated_at))
                .then_with(|| left.name.cmp(&right.name))
        });
        contacts
    }

    pub fn threads_snapshot(&self) -> Vec<MessageThread> {
        let mut threads = self.threads.clone();
        threads.sort_by(|left, right| {
            right
                .last_message_at
                .cmp(&left.last_message_at)
                .then_with(|| right.updated_at.cmp(&left.updated_at))
                .then_with(|| left.title.cmp(&right.title))
        });
        for thread in &mut threads {
            thread
                .messages
                .sort_by(|left, right| left.created_at.cmp(&right.created_at));
        }
        threads
    }

    pub fn mark_called(&mut self, phone: &str) {
        let now = now_ts();
        if let Some(contact) = self
            .contacts
            .iter_mut()
            .find(|contact| contact.phone == phone)
        {
            contact.note = String::from("Called now");
            contact.last_called_at = now;
            contact.updated_at = now;
            let _ = self.persist();
        }
    }

    pub fn ensure_direct_thread(&mut self, contact: &ContactRecord) -> usize {
        let now = now_ts();
        if let Some(index) = self
            .threads
            .iter_mut()
            .position(|thread| thread.title == contact.name)
        {
            if let Some(thread) = self.threads.get_mut(index) {
                thread.kind = ThreadKind::Direct;
                thread.updated_at = now;
            }
            let _ = self.persist();
            return index;
        }

        self.threads.push(MessageThread {
            title: contact.name.clone(),
            status: String::from("Direct"),
            preview: format!("Conversation with {}", contact.name),
            messages: vec![
                message(
                    &contact.name,
                    &format!("Direct line ready at {}.", contact.phone),
                    false,
                    now,
                ),
                message("You", "Draft channel opened from Contacts.", true, now + 1),
            ],
            created_at: now,
            last_message_at: now + 1,
            kind: ThreadKind::Direct,
            updated_at: now + 1,
        });

        if let Some(existing_contact) = self
            .contacts
            .iter_mut()
            .find(|existing_contact| existing_contact.phone == contact.phone)
        {
            existing_contact.last_message_at = now + 1;
            existing_contact.updated_at = now + 1;
        }

        let index = self.threads.len().saturating_sub(1);
        let _ = self.persist();
        index
    }

    pub fn append_outgoing_message(&mut self, thread_index: usize, body: &str) {
        let now = now_ts();
        if let Some(thread) = self.threads.get_mut(thread_index) {
            thread.messages.push(message("You", body, true, now));
            thread.preview = body.to_string();
            thread.status = String::from("Updated");
            thread.last_message_at = now;
            thread.updated_at = now;

            if thread.kind == ThreadKind::Direct {
                if let Some(contact) = self
                    .contacts
                    .iter_mut()
                    .find(|contact| contact.name == thread.title)
                {
                    contact.last_message_at = now;
                    contact.updated_at = now;
                }
            }

            let _ = self.persist();
        }
    }

    fn load_from_db(db_path: &Path) -> rusqlite::Result<Option<Self>> {
        let connection = Connection::open(db_path)?;
        migrate_schema(&connection)?;

        let contact_count: i64 =
            connection.query_row("SELECT COUNT(*) FROM contacts", [], |row| row.get(0))?;
        let thread_count: i64 =
            connection.query_row("SELECT COUNT(*) FROM threads", [], |row| row.get(0))?;

        if contact_count == 0 && thread_count == 0 {
            return Ok(None);
        }

        let contacts = {
            let mut statement = connection.prepare(
                "SELECT name, phone, note, created_at, last_called_at, last_message_at, updated_at
                 FROM contacts
                 ORDER BY last_called_at DESC, last_message_at DESC, updated_at DESC, id ASC",
            )?;
            let contacts = statement
                .query_map([], |row| {
                    Ok(ContactRecord {
                        name: row.get(0)?,
                        phone: row.get(1)?,
                        note: row.get(2)?,
                        created_at: row.get(3)?,
                        last_called_at: row.get(4)?,
                        last_message_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            contacts
        };

        let threads = {
            let mut statement = connection.prepare(
                "SELECT id, title, status, preview, created_at, last_message_at, thread_kind, updated_at
                 FROM threads
                 ORDER BY last_message_at DESC, updated_at DESC, id ASC",
            )?;
            let thread_rows = statement
                .query_map([], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, i64>(7)?,
                    ))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;

            let mut threads = Vec::with_capacity(thread_rows.len());
            for (
                thread_id,
                title,
                status,
                preview,
                created_at,
                last_message_at,
                thread_kind,
                updated_at,
            ) in thread_rows
            {
                let mut message_stmt = connection.prepare(
                    "SELECT author, body, outgoing, created_at, updated_at
                     FROM messages
                     WHERE thread_id = ?
                     ORDER BY created_at ASC, sort_index ASC, id ASC",
                )?;
                let messages = message_stmt
                    .query_map([thread_id], |row| {
                        Ok(MessageRecord {
                            author: row.get(0)?,
                            body: row.get(1)?,
                            outgoing: row.get::<_, i64>(2)? != 0,
                            created_at: row.get(3)?,
                            updated_at: row.get(4)?,
                        })
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?;

                threads.push(MessageThread {
                    title,
                    status,
                    preview,
                    messages,
                    created_at,
                    last_message_at,
                    kind: ThreadKind::from_db_value(&thread_kind),
                    updated_at,
                });
            }
            threads
        };

        Ok(Some(Self {
            contacts,
            db_path: db_path.to_path_buf(),
            threads,
        }))
    }

    fn persist(&self) -> rusqlite::Result<()> {
        let _ = init_parent_dir(&self.db_path);
        let mut connection = Connection::open(&self.db_path)?;
        migrate_schema(&connection)?;

        let transaction = connection.transaction()?;
        transaction.execute("DELETE FROM messages", [])?;
        transaction.execute("DELETE FROM threads", [])?;
        transaction.execute("DELETE FROM contacts", [])?;

        for contact in &self.contacts {
            transaction.execute(
                "INSERT INTO contacts (name, phone, note, created_at, last_called_at, last_message_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    contact.name,
                    contact.phone,
                    contact.note,
                    contact.created_at,
                    contact.last_called_at,
                    contact.last_message_at,
                    contact.updated_at
                ],
            )?;
        }

        for (thread_index, thread) in self.threads.iter().enumerate() {
            let thread_id = thread_index as i64 + 1;
            transaction.execute(
                "INSERT INTO threads (id, title, status, preview, created_at, last_message_at, thread_kind, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    thread_id,
                    thread.title,
                    thread.status,
                    thread.preview,
                    thread.created_at,
                    thread.last_message_at,
                    thread.kind.as_db_value(),
                    thread.updated_at
                ],
            )?;

            for (message_index, message) in thread.messages.iter().enumerate() {
                transaction.execute(
                    "INSERT INTO messages (thread_id, author, body, outgoing, sort_index, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        thread_id,
                        message.author,
                        message.body,
                        if message.outgoing { 1_i64 } else { 0_i64 },
                        message_index as i64,
                        message.created_at,
                        message.updated_at
                    ],
                )?;
            }
        }

        transaction.commit()
    }
}

fn migrate_schema(connection: &Connection) -> rusqlite::Result<()> {
    let current_version: i32 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    if current_version > SCHEMA_VERSION {
        return Err(rusqlite::Error::InvalidQuery);
    }

    for version in current_version + 1..=SCHEMA_VERSION {
        apply_migration(connection, version)?;
    }

    Ok(())
}

fn apply_migration(connection: &Connection, version: i32) -> rusqlite::Result<()> {
    match version {
        1 => migrate_to_v1(connection),
        2 => migrate_to_v2(connection),
        3 => migrate_to_v3(connection),
        _ => Err(rusqlite::Error::InvalidQuery),
    }
}

fn migrate_to_v1(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS contacts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            phone TEXT NOT NULL,
            note TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS threads (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            status TEXT NOT NULL,
            preview TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            thread_id INTEGER NOT NULL,
            author TEXT NOT NULL,
            body TEXT NOT NULL,
            outgoing INTEGER NOT NULL,
            sort_index INTEGER NOT NULL,
            FOREIGN KEY(thread_id) REFERENCES threads(id) ON DELETE CASCADE
        );

        PRAGMA user_version = 1;
        ",
    )
}

fn migrate_to_v2(connection: &Connection) -> rusqlite::Result<()> {
    let now = now_ts();
    connection.execute_batch(&format!(
        "
        ALTER TABLE contacts ADD COLUMN created_at INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE contacts ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0;

        ALTER TABLE threads ADD COLUMN created_at INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE threads ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0;

        ALTER TABLE messages ADD COLUMN created_at INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE messages ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0;

        UPDATE contacts
        SET created_at = CASE WHEN created_at = 0 THEN {now} ELSE created_at END,
            updated_at = CASE WHEN updated_at = 0 THEN {now} ELSE updated_at END;

        UPDATE threads
        SET created_at = CASE WHEN created_at = 0 THEN {now} ELSE created_at END,
            updated_at = CASE WHEN updated_at = 0 THEN {now} ELSE updated_at END;

        UPDATE messages
        SET created_at = CASE WHEN created_at = 0 THEN {now} ELSE created_at END,
            updated_at = CASE WHEN updated_at = 0 THEN {now} ELSE updated_at END;

        PRAGMA user_version = 2;
        "
    ))
}

fn migrate_to_v3(connection: &Connection) -> rusqlite::Result<()> {
    let now = now_ts();
    connection.execute_batch(&format!(
        "
        ALTER TABLE contacts ADD COLUMN last_called_at INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE contacts ADD COLUMN last_message_at INTEGER NOT NULL DEFAULT 0;

        ALTER TABLE threads ADD COLUMN last_message_at INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE threads ADD COLUMN thread_kind TEXT NOT NULL DEFAULT 'system';

        UPDATE contacts
        SET last_called_at = CASE
                WHEN note LIKE 'Called%' THEN updated_at
                ELSE last_called_at
            END,
            last_message_at = CASE
                WHEN last_message_at = 0 THEN created_at
                ELSE last_message_at
            END;

        UPDATE threads
        SET last_message_at = CASE
                WHEN last_message_at = 0 THEN updated_at
                ELSE last_message_at
            END,
            thread_kind = CASE
                WHEN thread_kind = 'system' AND title = 'Relay Bridge' THEN 'relay'
                WHEN thread_kind = 'system' AND title = 'Contacts Queue' THEN 'system'
                WHEN thread_kind = 'system' AND EXISTS (
                    SELECT 1 FROM contacts WHERE contacts.name = threads.title
                ) THEN 'direct'
                WHEN thread_kind = 'system' THEN 'group'
                ELSE thread_kind
            END;

        UPDATE contacts
        SET last_message_at = COALESCE(
            (
                SELECT MAX(threads.last_message_at)
                FROM threads
                WHERE threads.thread_kind = 'direct' AND threads.title = contacts.name
            ),
            CASE WHEN last_message_at = 0 THEN {now} ELSE last_message_at END
        );

        PRAGMA user_version = 3;
        "
    ))
}

fn init_parent_dir(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn store_db_path() -> PathBuf {
    let config_root = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));

    config_root
        .join("platinum-shell-gtk")
        .join("launcher-store.db")
}

fn contact(name: &str, phone: &str, note: &str, timestamp: i64) -> ContactRecord {
    ContactRecord {
        name: String::from(name),
        phone: String::from(phone),
        note: String::from(note),
        created_at: timestamp,
        last_called_at: 0,
        last_message_at: timestamp,
        updated_at: timestamp,
    }
}

fn message(author: &str, body: &str, outgoing: bool, timestamp: i64) -> MessageRecord {
    MessageRecord {
        author: String::from(author),
        body: String::from(body),
        outgoing,
        created_at: timestamp,
        updated_at: timestamp,
    }
}

fn now_ts() -> i64 {
    Utc::now().timestamp()
}

impl ThreadKind {
    fn as_db_value(self) -> &'static str {
        match self {
            ThreadKind::Direct => "direct",
            ThreadKind::Group => "group",
            ThreadKind::Relay => "relay",
            ThreadKind::System => "system",
        }
    }

    fn from_db_value(value: &str) -> Self {
        match value {
            "direct" => Self::Direct,
            "group" => Self::Group,
            "relay" => Self::Relay,
            _ => Self::System,
        }
    }
}
