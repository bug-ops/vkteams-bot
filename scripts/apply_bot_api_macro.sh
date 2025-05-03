#!/usr/bin/env bash
set -euo pipefail

# Скрипт автоматически заменяет декларации Request/Response и impl BotRequest на вызовы bot_api_method!
FILES=(
  src/api/chats/*.rs
  src/api/messages/*.rs
  src/api/files/*.rs
  src/api/events/*.rs
  src/api/myself/*.rs
)
for f in "${FILES[@]}"; do
  # пропускаем специальные модули
  [[ $f =~ (default|display|types|macros) ]] && continue
  echo "Патчим: $f"
  # вставляем импорт макроса после типов
  sed -i.bak '/use crate::api::types;/a use crate::api::macros::*;' "$f"
  # удаляем старый код Request*/Response* и impl BotRequest
  sed -i.bak '/pub struct Request/,/^}/d' "$f"
  sed -i.bak '/pub struct Response/,/^}/d' "$f"
  sed -i.bak '/impl BotRequest/,/^}/d' "$f"
  # добавляем шаблон вызова макроса в конец файла
  cat << 'EOF' >> "$f"

// TODO: заменить плейсхолдеры на реальные данные метода
bot_api_method! {
    method   = "<METHOD_PATH>",
    request  = <RequestStruct> { /* поля */ },
    response = <ResponseStruct> { /* поля */ },
}
EOF
  rm -f "$f.bak"
done

echo "Готово. Проверьте TODO в каждом файле и подставьте правильные пути и поля." 