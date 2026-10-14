# **app Timer**
## Описание
"**app Timer**" — простое приложение для отслеживания времени. Оно позволяет пользователям добавлять, удалять и редактировать элементы, а также отслеживать общее время, проведенное за каждой игрой. Все данные сохраняются локально в формате JSON.  

### Особенности
- Отслеживание времени : Показывает общее время, проведенное за каждой добавленной задачей.
- Добавление элементов : Возможность добавлять новые элементы в список.
- Удаление элемнтов
- Редактирование времени : Ручное изменение времени для любого элемента.
- Пауза/Старт : Возможность поставить таймер на паузу или возобновить его.
- Сохранение данных : Все данные (названия элементов и затраченное время) сохраняются в файл apps.json.
- Простой интерфейс : Легкое управление с помощью кнопок и меню.  

![enter image description here](https://github.com/gillenormand/TimerApp-rust-port/blob/main/readme_files/img1.jpg)

# Rust Portable with MinGW

## Downloads

1. **MinGW (UCRT runtime)**: [winlibs.com](https://winlibs.com/) – Latest Release
2. **Rustup**: [rustup.rs](https://rustup.rs/) – `rustup-init.exe`
3. **VS Code**: [code.visualstudio.com](https://code.visualstudio.com/download) – ZIP version
4. **bat scripts**: [github](https://github.com/gillenormand/TimerApp-rust-port/blob/main/readme_files/RustPortable_batScripts.7z) and place them into your desired portable folder

## Installation Steps

1. Run `create_folderSystem.bat` to create the directory structure.
2. Place `rustup-init.exe` in the root folder (next to the scripts).
3. Run `INSTALL.bat`.
4. Extract VS Code into the `vscode` folder.
5. Create a `data` folder inside the `vscode` folder.
6. Run `rust_activation_enc.bat`.
7. Open VS Code and install extensions:
   - **rust-analyzer**
   - **CodeLLDB**
