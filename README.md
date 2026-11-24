# Fenrae Eclipse

Natywna aplikacja Windows w Rust. Aby uruchomić grę, otwórz `Fenrae Eclipse.exe`.

1. Wybierz **Graj → Pojedynek 1 na 1 z botem → Rozpocznij mecz**.
2. Kliknij ikonę bohatera na wspólnej planszy wyboru, a następnie **Rozpocznij pojedynek**. Możesz również przechodzić między ikonami klawiszem Tab i zatwierdzić wybór klawiszem Enter.
3. Kliknij teren, aby się ruszyć, lub wroga, aby atakować. **S** zatrzymuje ruch i atak. **T** wybiera wrogiego bohatera lub miniona najbliżej kursora i rozpoczyna atak. **A** pokazuje zasięg podstawowego ataku do następnego kliknięcia. Celuj kursorem i używaj **Q**, **W**, **E** i **R**. Taran i Nera korzystają z **Q** oraz **E**.
4. Przesuń kursor do krawędzi widoku, żeby przesuwać kamerę po moście. Przytrzymaj spację, żeby kamera śledziła bohatera. **V** przełącza kąt widoku. Kursor pozostaje w obszarze aktywnego meczu. **Alt+Tab** zwalnia go po przełączeniu aplikacji. Kliknij minimapę w prawym dolnym rogu, aby przesunąć kamerę.
5. Eskortuj fale minionów, niszcząc kolejno trzy wieże wroga. Po upadku ostatniej zaatakuj Nexus. Jego zniszczenie kończy mecz.

## Statystyki walki

Parametry bohaterów są w modułach Rust w folderze `src/characters/`. AD to obrażenia podstawowego ataku. U Tarana wpływa też na fizyczne obrażenia Q. AP zwiększa magiczne obrażenia Q Nery. Pancerz zmniejsza obrażenia fizyczne, a odporność magiczna obrażenia magiczne. Osłona Tarana dodatkowo zmniejsza oba rodzaje obrażeń.

Szybkość ataku podawana jest w atakach na sekundę. Wszechwampiryzm leczy od faktycznie odebranego wrogowi zdrowia po uwzględnieniu odporności. Działa na postacie i jednostki, z pominięciem wież.

## Bohaterowie z kart

Aelor, Arxen, Aurelia, Helvara, Kaelith, Luness, Nyxalune, Selune, Solviana, Veyrion, Zyraen i Elorim są odblokowani od początku. Każda z tych postaci ma portret z karty, umiejętność pasywną i cztery aktywne. Na ekranie wyboru wszystkie 14 postaci ma własną ikonę.

W pojedynku efekty wsparcia obejmują wybranego sojusznika, własną postać lub jednostki z sojuszniczej fali. Wybierz cel kursorem. Do wskrzeszenia Selune wskaż miejsce, w którym sojusznicza jednostka poległa w ciągu ostatnich 3 sekund. Jeśli trafiony sojusznik żyje i ma poniżej 25% zdrowia, R natychmiast go leczy i osłania.

## Kod

| Folder | Zawartość |
| --- | --- |
| `src/main_menu/` | Okno menu, nawigacja i uruchamianie meczu. |
| `src/characters/` | Statystyki i parametry Tarana oraz Nery, a także katalog bohaterów z kart w `roster.rs`. |
| `src/inventory/` | Stan początkowego ekwipunku i lista odblokowanych postaci. |
| `src/gameplay/` | Symulacja: walka, bot, fale jednostek, wieże i wynik. Osobne pliki obsługują okno oraz rysowanie areny. |
| `src/ui/` | Wspólne kontrolki Windows, kolory, czcionki i ładowanie grafiki. |
| `assets/` | Ilustracje oraz zasoby ikony i metadanych aplikacji. |

`src/main.rs` jest punktem startowym aplikacji i wywołuje okno menu.

## Kompilacja

Wymagane są Rust MSVC x64 (1.88 lub nowszy), Visual Studio Build Tools z C++ oraz Windows SDK. Uruchom w katalogu projektu:

```powershell
cargo build --release --locked
```

Plik wynikowy to `target/release/moba-prototyp.exe`. Plik `Fenrae Eclipse.exe` w katalogu projektu jest jego kopią. Kompilacja nie podpisuje pliku cyfrowo.

## Grafika i ikona

Menu i ikona wywodzą się z dostarczonej grafiki Fenrae Eclipse. Plansza wyboru korzysta z ilustracji bohaterów.

- Scena meczu powstaje z geometrii 3D i jest rysowana programowo z użyciem bufora głębokości. Kod kamery, terenu, modeli i efektów znajduje się w `src/gameplay/window/scene/` oraz `scene.rs`.
- Portrety: dwóch bohaterów na jednej planszy, Taran w kutej zbroi z żarem i Nera w ametystowym kapturze ze srebrnym łukiem, z oświetleniem zaćmienia.

Okna korzystają z bitmap osadzonych w programie. Windows odczytuje ikonę oraz nazwę produktu z zasobów pliku wykonywalnego. Po zmianie `assets/fenrae.ico` odtwórz zasób poleceniem poniżej w terminalu Visual Studio i ponownie zbuduj grę.

```powershell
rc /nologo /fo assets\fenrae.res assets\fenrae.rc
cargo build --release --locked
```

## Serwer meczu i kod umiejętności

Każdy pojedynek uruchamia lokalny serwer w osobnym wątku gry. Serwer aktualizuje symulację 60 razy na sekundę, steruje botem i publikuje stan dla renderera. Po zakończeniu sesji połączenie jest zamykane.

Dane bohatera są w `src/characters/<postać>.rs`, a wykonanie jego aktywnych umiejętności w `src/characters/<postać>/abilities.rs`. Wspólne reguły trafień, stref, efektów okresowych i pasywnych należą do symulacji.

Kliknięcie ikony lub pojedyncze naciśnięcie skrótu wysyła do serwera przez TCP numer umiejętności i współrzędne celu. Połączenie jest lokalne (127.0.0.1). Serwer wybiera kod według postaci gracza i sprawdza stan meczu, życie, ogłuszenie, odnowienie oraz warunki konkretnej umiejętności. Klient rysuje otrzymany stan. Ruch, zatrzymanie i rozkaz ataku korzystają z tego samego połączenia.
