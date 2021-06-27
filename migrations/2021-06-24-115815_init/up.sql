-- Your SQL goes here

create table tomatos (
	id               INTEGER PRIMARY KEY NOT NULL, -- alias for rowid          
	inventory_id     INT NOT NULL,
	task_id          INT NOT NULL,
	start_time       BIGINT NOT NULL,
	end_time         BIGINT NOT NULL
);


create table tasks (
	id               INTEGER PRIMARY KEY NOT NULL,
	inventory_id	 INT NOT NULL,
	name             TEXT NOT NULL,
	spent_minutes    BIGINT NOT NULL,
	create_at        BIGINT NOT NULL,
	notes            TEXT
);

insert into tasks values (0, 0, "未指定", 0, 0, NULL);


create table inventory (
	id               INTEGER PRIMARY KEY NOT NULL,
	name             TEXT NOT NULL,
	color			 INT NOT NULL
);

insert into inventory values (0, "未指定", 0);

