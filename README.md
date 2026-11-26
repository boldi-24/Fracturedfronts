# Fractured Front

Natywna aplikacja Windows w Rust. Aby uruchomić grę, otwórz `ff.exe`.
Aby uruchomić tą grę jako że nie jest zweryfikowana przez windows trzeba wyłączyć smart app control
aby to zrobić 
• Otwórz Zabezpieczenia Windows (wyszukaj w menu Start).
• Przejdź do sekcji Kontrola aplikacji i przeglądarki.
• Kliknij Ustawienia funkcji Inteligentna kontrola aplikacji.
• Zmień przełącznik na Wyłączone.
## zmiany w kodzie
po każdej zmianie w kodzie trzeba skompilować na nowo plik ff.exe robi się to poleceniem 
cargo run --release
jeśli nie działa dalej należy wykonać te komendy 
$env:Path = "$env:USERPROFILE\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin;$env:Path"
cd "folder w którym znajduje się fractuder front"
cargo run --release
w terminalu w folderze fractured front
jeśli te zmiany mają zostać zapisane trzeba wprowadzić polecenie 
Copy-Item .\target\release\moba-prototyp.exe .\ff.exe -Force

## Wspólne umiejętności obu postaci

Q wystrzeliwuje trzy strzały, W oznacza cel, E wykonuje odskok, a R tworzy obszar deszczu strzał. Kolejne ataki na ten sam cel zyskują premię do obrażeń.

## Kod

| Folder | Zawartość |
| --- | --- |
| `src/main_menu/` | Okno menu, nawigacja i uruchamianie meczu. |
| `src/characters/` | Profile obu postaci i kod wspólnych umiejętności. |
| `src/inventory/` | Stan początkowego ekwipunku i lista odblokowanych postaci. |
| `src/gameplay/` | Symulacja: walka, bot, generatory i wynik. Osobne pliki obsługują okno oraz rysowanie areny. |
| `src/ui/` | Wspólne kontrolki Windows, kolory, czcionki i ładowanie grafiki. |
| `assets/` | Ilustracje oraz zasoby ikony i metadanych aplikacji. |

`src/main.rs` jest punktem startowym aplikacji i wywołuje okno menu.

## Kompilacja

Wymagane są Rust MSVC x64 (1.88 lub nowszy), Visual Studio Build Tools z C++ oraz Windows SDK. Uruchom w katalogu projektu:

```powershell
cargo build --release --locked
```

Plik wynikowy to `target/release/moba-prototyp.exe`. Plik `ff.exe` w katalogu projektu jest jego kopią. Kompilacja nie podpisuje pliku cyfrowo.

## Grafika i ikona

Menu i ikona wywodzą się z dostarczonej grafiki Fractured Front. Plansza wyboru korzysta z ilustracji bohaterów.

- Scena meczu powstaje z geometrii 3D i jest rysowana programowo z użyciem bufora głębokości. Kod kamery, terenu, modeli i efektów znajduje się w `src/gameplay/window/scene/` oraz `scene.rs`.

Okna korzystają z bitmap osadzonych w programie. Windows odczytuje ikonę oraz nazwę produktu z zasobów pliku wykonywalnego. Po zmianie `assets/ff.ico` odtwórz zasób poleceniem poniżej w terminalu Visual Studio i ponownie zbuduj grę.

```powershell
rc /nologo /fo assets\ff.res assets\ff.rc
cargo build --release --locked
```

## Serwer meczu i kod umiejętności

Każdy pojedynek uruchamia lokalny serwer w osobnym wątku gry. Serwer aktualizuje symulację 60 razy na sekundę, steruje botem i publikuje stan dla renderera. Po zakończeniu sesji połączenie jest zamykane.

Dane bohatera są w `src/characters/<postać>.rs`, a wykonanie jego aktywnych umiejętności w `src/characters/<postać>/abilities.rs`. Wspólne reguły trafień, stref, efektów okresowych i pasywnych należą do symulacji.

Kliknięcie ikony lub pojedyncze naciśnięcie skrótu wysyła do serwera przez TCP numer umiejętności i współrzędne celu. Połączenie jest lokalne (127.0.0.1). Serwer wybiera kod według postaci gracza i sprawdza stan meczu, życie, odnowienie oraz warunki konkretnej umiejętności. Klient rysuje otrzymany stan. Ruch, zatrzymanie i rozkaz ataku korzystają z tego samego połączenia.
