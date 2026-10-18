# Fixture SQLite para pruebas manuales

`demo.sqlite` contiene datos ficticios y está pensado para explorar consultas en Tinker. No se conecta a Laravel automáticamente: el archivo debe copiarse o montarse en el proyecto Laravel que se ejecute.

## Esquema

- `users`: cinco usuarios ficticios, con estados activo/inactivo.
- `posts`: seis publicaciones asociadas a usuarios mediante `user_id`.
- `comments`: cinco comentarios asociados a publicaciones mediante `post_id`.

El esquema declara claves foráneas. Si se hacen escrituras, cada conexión SQLite debe activarlas con `PRAGMA foreign_keys = ON`.

## Uso con Tinker

Desde un proyecto Laravel, apunta temporalmente la conexión al archivo mediante una ruta accesible para ese proceso o contenedor:

```php
config([
    'database.default' => 'sqlite',
    'database.connections.sqlite.database' => '/ruta/a/demo.sqlite',
]);
DB::purge('sqlite');
DB::reconnect('sqlite');

DB::table('users')->get();
```

En Docker, `/ruta/a/demo.sqlite` debe corresponder a un archivo montado dentro del contenedor. La base solo es un fixture SQL: no incluye modelos ni migraciones de Laravel.
