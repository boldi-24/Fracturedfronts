# Fractured Front

Natywna aplikacja Windows w Rust. Aby uruchomić grę, otwórz `ff.exe`.

1. Wybierz **Graj → Pojedynek 1 na 1 z botem → Rozpocznij mecz**.
2. Wybierz **Rozpocznij pojedynek**. Wybierz Postać 1 lub Postać 2. Bot gra drugim bohaterem.
3. Kliknij teren, aby się ruszyć, lub wroga, aby atakować. **S** zatrzymuje ruch i atak. **T** wybiera wrogiego bohatera najbliżej kursora i rozpoczyna atak. **A** pokazuje zasięg podstawowego ataku do następnego kliknięcia. Celuj kursorem i używaj **Q**, **W**, **E** i **R**.
4. Przesuń kursor do krawędzi widoku, żeby przesuwać kamerę po mapie. Przytrzymaj spację, żeby kamera śledziła bohatera. **V** przełącza kąt widoku. Kursor pozostaje w obszarze aktywnego meczu. **Alt+Tab** zwalnia go po przełączeniu aplikacji. Kliknij minimapę w prawym dolnym rogu, aby przesunąć kamerę.
5. Walcz w kompleksie pomieszczeń i korytarzy 2560 × 1536 z planu `assets/mapa.png`. Zniszczenie generatora Nexus przeciwnika kończy mecz. Oba generatory są dostępne od początku.

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
