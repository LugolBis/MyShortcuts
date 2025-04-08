# MyShortcuts

**MyShortcuts** is an interactive terminal user interface (TUI) application for managing shell command and databases connections.

The tool offers the following features :
- Save database connections and shell command
- Edit database connections and shell command
- Formats your configurations and generates a shell command (see [Available Scheme](https://github.com/LugolBis/MyShortcuts/new/main?filename=README.md#available-scheme))
- Open a new terminal and execute a shell command on it

<br>

## Getting started
### Linux
Clone the repository :
```BashScript
$ git clone https://github.com/LugolBis/MyShortcuts.git
```
Build the project :
```BashScript
$ cd etc/MyShortcuts && cargo build
```
Launch it :
```BashScript
$ cargo run
```

### Windows
Clone the repository :
```BashScript
$ git clone https://github.com/LugolBis/MyShortcuts.git
```
Configure the permissions :
```BashScript
$ # Not already supported.
```
Build the project :
```BashScript
$ cd etc/MyShortcuts && cargo build
```
Launch it :
```BashScript
$ cargo run
```

## Available Scheme
**MyShortcuts** integrate predefined schemes for the databases connection. These schemes help you to adding and editing a new database connection by provide you the configuration needed by any of them.
Moreover these schemes are used to format your configuration and generate a shell command with your arguments and the correct flags.
||System|Kind|
|:-|:-:|:-:|
|![Oracle](https://img.shields.io/badge/Oracle-F80000?style=for-the-badge&logo=oracle&logoColor=white)|Oracle|Database connection|
|![MySQL](https://img.shields.io/badge/mysql-4479A1.svg?style=for-the-badge&logo=mysql&logoColor=white)|MySQL|Database connection|
|![MariaDB](https://img.shields.io/badge/MariaDB-003545?style=for-the-badge&logo=mariadb&logoColor=white)|MariaDB|Database connection|
|![Postgres](https://img.shields.io/badge/postgres-%23316192.svg?style=for-the-badge&logo=postgresql&logoColor=white)|PostgreSQL|Database connection|
|![SQLite](https://img.shields.io/badge/sqlite-%2307405e.svg?style=for-the-badge&logo=sqlite&logoColor=white)|SQLite|Database connection|
|![Redis](https://img.shields.io/badge/redis-%23DD0031.svg?style=for-the-badge&logo=redis&logoColor=white)|Redis|Database connection|
|![MongoDB](https://img.shields.io/badge/MongoDB-%234ea94b.svg?style=for-the-badge&logo=mongodb&logoColor=white)|MongoDB|Database connection|
|![Neo4J](https://img.shields.io/badge/Neo4j-008CC1?style=for-the-badge&logo=neo4j&logoColor=white)|Neo4j|Database connection|
|![Custom](https://img.shields.io/badge/custom-a08021?style=for-the-badge&logo=custom&logoColor=ffcd34)|Custom|Shell command|

<br>

> [!NOTE]
> **Custom** scheme support any shell command.

> [!TIP]
> Database connection schemes allow you to specify a script path to run in the database (this option runs your script in your database and does not persist the connection).

## Compatibility
|Shell|Supported|
|:-:|:-:|
|BashScript|✅​|
|PowerShell|🕑​|

## Privacy
**MyShortcuts** save your data locally with SQLite, don't worry about saving your passwords !
