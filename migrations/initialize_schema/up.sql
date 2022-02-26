CREATE TABLE 'wallet' (
  'id' INTEGER DEFAULT NULL PRIMARY KEY AUTOINCREMENT,
  'uuid' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'address_type' SMALLINT NOT NULL  DEFAULT NULL,
  'receive_descriptor' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'receive_address_index' INTEGER NOT NULL  DEFAULT NULL,
  'receive_address' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'change_descriptor' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'change_address_index' INTEGER NOT NULL  DEFAULT NULL,
  'change_address' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'required_signatures' INTEGER NOT NULL  DEFAULT NULL,
  'creation_time' DATETIME NOT NULL  DEFAULT 'NULL'
);

CREATE TABLE 'cosigner' (
  'id' INTEGER DEFAULT NULL PRIMARY KEY AUTOINCREMENT,
  'uuid' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'cosigner_type' SMALLINT DEFAULT NULL,
  'email' VARCHAR(50) DEFAULT NULL,
  'wallet_id' INTEGER DEFAULT NULL REFERENCES 'wallet' ('id')
);

CREATE TABLE 'psbt' (
  'id' INTEGER DEFAULT NULL PRIMARY KEY AUTOINCREMENT,
  'uuid' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'data' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'cosigner_id' INTEGER DEFAULT NULL REFERENCES 'cosigner' ('id'),
  'wallet_id' INTEGER DEFAULT NULL REFERENCES 'wallet' ('id')
);

CREATE TABLE 'xpub' (
  'id' INTEGER DEFAULT NULL PRIMARY KEY AUTOINCREMENT,
  'uuid' MEDIUMTEXT DEFAULT NULL,
  'derivation_path' MEDIUMTEXT DEFAULT NULL,
  'fingerprint' MEDIUMTEXT DEFAULT NULL,
  'data' MEDIUMTEXT NOT NULL  DEFAULT 'NULL',
  'cosigner_id' INTEGER DEFAULT NULL REFERENCES 'cosigner' ('id'),
  'wallet_id' INTEGER DEFAULT NULL REFERENCES 'wallet' ('id')
);

CREATE TABLE 'xprv' (
  'id' INTEGER DEFAULT NULL PRIMARY KEY AUTOINCREMENT,
  'uuid' MEDIUMTEXT DEFAULT NULL,
  'fingerprint' MEDIUMTEXT DEFAULT NULL,
  'mnemonic' MEDIUMTEXT DEFAULT NULL,
  'data' MEDIUMTEXT DEFAULT NULL,
  'cosigner_id' INTEGER DEFAULT NULL REFERENCES 'cosigner' ('id'),
  'wallet_id' INTEGER DEFAULT NULL REFERENCES 'wallet' ('id')
);

CREATE INDEX 'wallet_uuid_idx' ON 'wallet' ('uuid');
CREATE INDEX 'cosigner_uuid_idx' ON 'cosigner' ('uuid');
CREATE INDEX 'cosigner_wallet_idx' ON 'cosigner' ('wallet_id');
CREATE INDEX 'psbt_uuid_idx' ON 'psbt' ('uuid');
CREATE INDEX 'psbt_cosigner_idx' ON 'psbt' ('cosigner_id');
CREATE INDEX 'psbt_wallet_idx' ON 'psbt' ('wallet_id');
CREATE INDEX 'xpub_uuid_idx' ON 'xpub' ('uuid');
CREATE INDEX 'xpub_cosigner_idx' ON 'xpub' ('cosigner_id');
CREATE INDEX 'xpub_wallet_idx' ON 'xpub' ('wallet_id');
CREATE INDEX 'xprv_uuid_idx' ON 'xprv' ('uuid');
CREATE INDEX 'xprv_cosigner_idx' ON 'xprv' ('cosigner_id');
CREATE INDEX 'xprv_wallet_idx' ON 'xprv' ('wallet_id');
