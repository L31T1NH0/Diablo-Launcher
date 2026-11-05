# Catálogo de opções

`defaults-1.5.5.ini` contém as 190 preferências persistidas pelo DevilutionX 1.5.5 em Linux: opções gráficas, áudio, idioma, jogabilidade, inicialização, demonstração, rede, mensagens, teclado e controle.

Referências oficiais:

- [Source/options.cpp na versão 1.5.5](https://github.com/diasurgical/DevilutionX/blob/1.5.5/Source/options.cpp): categorias, nomes INI, padrões e listas de valores.
- [Source/options.h na versão 1.5.5](https://github.com/diasurgical/DevilutionX/blob/1.5.5/Source/options.h): enumerações de vídeo, inicialização e números flutuantes.
- [Source/engine/sound_defs.hpp](https://github.com/diasurgical/DevilutionX/blob/1.5.5/Source/engine/sound_defs.hpp): intervalo do volume (-1600 a 0).

O catálogo serve para mostrar opções mesmo quando o INI ainda não contém todas as chaves. Ele não substitui o arquivo do usuário. Os valores existentes têm prioridade; novas chaves de versões/mods também são incorporadas à interface. Apenas campos efetivamente editados são gravados.

`LastSinglePlayerHero`, `LastMultiplayerHero` e `SItem` são estado de personagens/saves e permanecem intactos. `GameMode.Game` é controlado pela opção de expansão do launcher e pelos argumentos de execução. Opções específicas de outras plataformas não fazem parte do catálogo Linux. Idiomas ou recursos opcionais podem exigir os arquivos correspondentes da instalação do jogo.
