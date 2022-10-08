CREATE TABLE 'cosigner' (
  'id' INTEGER NOT NULL  DEFAULT NULL PRIMARY KEY AUTOINCREMENT,
  'uuid' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'type' SMALLINT NOT NULL  DEFAULT NULL,
  'email_address' VARCHAR(50) NOT NULL  DEFAULT 'NULL',
  'xpub' MEDIUMTEXT(120) DEFAULT NULL,
  'xprv' MEDIUMTEXT DEFAULT NULL,
  'creation_time' DATETIME NOT NULL  DEFAULT 'NULL',
  'wallet_uuid' MEDIUMTEXT DEFAULT NULL
);

CREATE TABLE 'wallet' (
  'id' INTEGER NOT NULL  DEFAULT NULL PRIMARY KEY AUTOINCREMENT,
  'uuid' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'address_type' SMALLINT NOT NULL  DEFAULT NULL,
  'network' SMALLINT NOT NULL  DEFAULT NULL,
  'receive_descriptor' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'receive_descriptor_watch_only' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'receive_address_index' BIGINT NOT NULL  DEFAULT NULL,
  'receive_address' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'change_descriptor' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'change_descriptor_watch_only' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'change_address_index' BIGINT NOT NULL  DEFAULT NULL,
  'change_address' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'required_signatures' SMALLINT NOT NULL  DEFAULT NULL,
  'balance' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'creation_time' DATETIME NOT NULL  DEFAULT 'NULL'
);

CREATE TABLE 'psbt' (
  'id' INTEGER NOT NULL  DEFAULT NULL PRIMARY KEY AUTOINCREMENT,
  'uuid' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'base64' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'creation_time' DATETIME NOT NULL  DEFAULT 'NULL',
  'wallet_uuid' MEDIUMTEXT NOT NULL  DEFAULT 'NULL'
);

CREATE INDEX 'cosigner_uuid_idx' ON 'cosigner' ('uuid');
CREATE INDEX 'wallet_uuid_idx' ON 'cosigner' ('wallet_uuid');
CREATE INDEX 'email_address_idx' ON 'cosigner' ('email_address');
CREATE INDEX 'xpub_idx' ON 'cosigner' ('xpub');
CREATE INDEX 'wallet_uuid_idx' ON 'wallet' ('uuid');
CREATE INDEX 'wallet_receive_descriptor_watch_only_idx' ON 'wallet' ('receive_descriptor_watch_only');
CREATE INDEX 'wallet_uuid_idx' ON 'psbt' ('wallet_uuid');
CREATE INDEX 'psbt_uuid_idx' ON 'psbt' ('uuid');
