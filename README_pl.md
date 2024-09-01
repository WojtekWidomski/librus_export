# LIBRUS Synergia Message Export

[English readme](README.md)

Ten program umożliwia pobranie wszystkich wiadomości z konta Librus Synergia i zapisanie ich w formacie JSON.

## Funkcje
- Eksportowanie ze wszystkich folderów, nawet z archiwum
- Zapisywanie dużych grup odbiorców do oddzielnego pliku
- Łatwe w obsłudze CLI (terminalowy interfejs) z paskami postępu z biblioteki `indicatif`
- Napisane w Ruscie

## Wyjście
Wyeksportowane dane są zapisywane w folderze `export_imie_nazwisko`
Powstają następujące pliki

- `messages_nazwa_folderu.json` (dla każdego folderu wiadomości) - wszystkie wiadomości w folderze
- `groups.json` - Grupy odbiorców

## Obsługa
Pobierz najnowszą wersję z [GitHub Releases](https://github.com/WojtekWidomski/librus_export/releases).  
Na Windowsie da się uruchomić program klikając ikonkę, ale zalecane jest uruchamianie z linii poleceń, żeby zobaczyć, co się stało w razie błędu.

Można też sklonować to repozytorium i skompilować i uruchomić program poleceniem `cargo run --release`.

Program zapyta, ile osób ma być uznane za dużą grupę. Małe grupy odbiorców zostaną zapisane w tym samym pliku, co wiadomości. Duże będą zapisane w tablicy w oddzielnym pliku, a w pliku z wiadomościami będzie tylko indeks w tamtej tablicy. Domyślnie 10.

Ta funkcja jest przydatna, bo niektórzy nauczyciele bardzo często wysyłają maile do wszystkich uczniów w szkole.

Potem program zapyta o nazwę użytkownika i hasło. Potem będzie trzeba wybrać, które foldery zapisać. Domyślnie wszystkie są wybrane.

## Użyte technologie
- Język programowania Rust
- W pliku [Cargo.toml](Cargo.toml) są używane biblioteki
- Mimo, że [to repozytorium](https://github.com/kbaraniak/librus-api/) nie jest zależnością tego programu, to patrzyłem na nie pisząc kod uwierzytelniania.
