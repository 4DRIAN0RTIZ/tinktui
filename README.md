# TinkTui

**Laravel Tinker, made comfortable.** A terminal interface for writing Eloquent queries, running them, and viewing results in clean tables—without fighting the REPL.

```
┌ History ────────────┐┌ PHP code · docker:my-app ───────────────────────────┐
│▶ User::latest()...  ││ 1  User::latest()->take(5)->get()                   │
│  Order::where(...   ││                                                     │
│  $h = Hardware::... ││                                                     │
│                     │└─────────────────────────────────────────────────────┘
│                     │┌ Result ✓ 0.67s · 5 rows · cols 1–6/22 · table ──────┐
│                     ││   id   username   email               enabled       │
│                     ││ ▶ 18   ldare      gina20@example.org  true          │
│                     ││   19   isabel     ms@example.net      true          │
│                     ││   16   keebler    zk@example.net      false         │
└─────────────────────┘└─────────────────────────────────────────────────────┘
```

## Why?

Tinker is great for exploring a Laravel application's data, but it has a few rough edges:

- Writing multi-line queries in a REPL is uncomfortable.
- A collection of models becomes a wall of hard-to-read text.
- When an app runs in Docker, you have to type the complete `docker exec ...` command every time.

**TinkTui** solves this: write in an editor, run with one key, and see the result in a table.

Read the full documentation at [tinktui.cuevaneander.tech](https://tinktui.cuevaneander.tech).

## Features

- **Write and run** with `Ctrl+R` or `F5`. The result appears below along with its execution time.
- **View collections as tables.** Each record becomes a row and every attribute a column. A single model is shown as a *field → value* list.
- **Keep sensitive data safe.** Tables honor your models' `$hidden` attributes, so passwords and tokens stay out of view.
- **Switch to the classic view** with `t`. It is regular Tinker output, with syntax highlighting.
- **Use your favorite editor.** `Ctrl+E` opens the current code in your editor (Neovim by default), with your usual completion and tooling. Close it to return with the code ready.
- **Recall previous queries.** Every executed snippet is stored in history, including between sessions.
- **Work with or without Docker.** Run against a container or a local Laravel project.

## Requirements

- [Rust](https://rustup.rs) to build it.
- A **Laravel** project with Tinker, which Laravel includes by default.
- **Docker** when the application runs in a container, or **PHP** when it runs on your machine.

## Installation

From the project directory:

```bash
cargo install --path .
```

The `tinker-tui` command is then available in your terminal.

## Usage

**For an app running in Docker**, provide the container name:

```bash
tinker-tui --container my-laravel-app
```

**For a local app**, enter the project directory and run:

```bash
tinker-tui
```

**To avoid entering the container every time**, add it to `.bashrc` or `.zshrc`:

```bash
export TINKER_TUI_CONTAINER=my-laravel-app
```

After that, just run `tinker-tui`.

## Keyboard shortcuts

**At any time**

| Key | Action |
|---|---|
| `Ctrl+R` / `F5` | Run the code |
| `Ctrl+E` | Open the code in your editor |
| `Ctrl+L` | Clear the result |
| `Ctrl+C` | Exit |

**Move between panels**

| Key | Action |
|---|---|
| `Esc` | Leave the editor |
| `i` | Return to the editor |
| `Tab` | Switch between history and result |
| `q` | Exit |

**In history**

| Key | Action |
|---|---|
| `j` / `k` | Move |
| `Enter` | Load the query into the editor |
| `d` | Delete the query |

**In the result**

| Key | Action |
|---|---|
| `t` | Switch between table and classic view |
| `j` / `k` | Move through rows |
| `h` / `l` | Move through columns |
| `0` / `$` | First / last column |
| `g` / `G` | Jump to start / end |

## Getting started examples

```php
// How many users are there?
User::count()

// The latest five, with only the fields you need
User::latest()->take(5)->get(['id', 'email', 'created_at'])

// Only email addresses
User::pluck('email')

// Multi-line snippets: the result from the final expression is displayed
$admins = User::where('role', 'admin')->get();
$admins->count()
```

## Tips

- **Every line should end with `;`, except the final one.** You may omit the final semicolon because the app adds it for you.
- **Each run starts from scratch.** A variable created in one run does not exist in the next. Put everything you need in the same snippet.
- **Request only the columns you need.** Tables with many attributes are easier to navigate with `->get(['id', 'name', ...])`.
- **Tinker runs real code against your database.** A `delete()` or `update()` takes effect immediately. Use it carefully, especially in production.
