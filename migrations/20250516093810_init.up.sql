CREATE TABLE willow_clients (
	id INTEGER NOT NULL,
	mac_addr VARCHAR NOT NULL,
	label VARCHAR NOT NULL,
	PRIMARY KEY (id),
	UNIQUE (mac_addr)
);

CREATE TABLE willow_config (
	id INTEGER NOT NULL,
	config_type VARCHAR(8) NOT NULL,
	config_name VARCHAR NOT NULL,
	config_namespace VARCHAR(4),
	config_value VARCHAR,
	PRIMARY KEY (id),
	UNIQUE (config_type, config_name)
);
