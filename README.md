# Acaboulette

Un petit jeu d'arcade (gore désactivable) pour Packarch, écrit en Rust avec
[macroquad](https://macroquad.rs), dans l'esprit de *Paf le Bengal*.

**Acaboulette**, un gros matou roux, traverse Bordeaux au pas de course et
mange ou écrabouille tout ce qui passe : les pigeons de la Bourse, les rats
de la Garonne, les chiens… C'est tarpin gore.

Tout est dessiné par le code : aucune image ni aucun son à charger.

Au démarrage, une fenêtre rappelle la licence (GPL-3.0) : **Accepter**
(ou Entrée) lance le jeu, **Refuser** (ou Échap) le quitte.

## Commandes

- **Espace / clic / flèche haut** : sauter (double saut ; maintenir = plus haut)
- **Troisième saut** (en l'air, après le double saut) : Acaboulette sort un
  petit **deltaplane** et plane un moment. Chaque clic redresse le nez et le
  fait remonter un peu, de moins en moins : la gravité finit toujours par
  gagner. Le plat ventre replie l'aile.
- **Flèche bas / S / clic droit** (en l'air) : **plat ventre**, l'onde de choc
  écrabouille tout autour
- **G** : activer / désactiver le gore
- **Échap** : quitter

## Le jeu

- **Pigeons, rats, chiens** : aucun danger, c'est à manger ! En courant
  dedans il les avale, en tombant dessus il les écrabouille. Plus il en
  mange, plus ça rapporte et plus il **grossit** (plats ventre plus larges,
  points multipliés).
- **Mouettes** : elles volent tout là-haut, hors de portée d'un double saut.
  On ne les attrape qu'en **deltaplane** (il faut souvent redresser le nez
  pour monter jusqu'à elles), et elles rapportent gros.
- **Combos** : enchaîner les bestioles fait monter le combo ; en avoir
  plusieurs **d'un coup** (plat ventre dans une bande de pigeons…) donne un
  gros bonus.
- **Barrage de CRS** : le **bonus**. Ils ne font rien, ils valdinguent
  (casque et matraque en l'air, sans gore) et rapportent gros.
- **Ce qui fait mal** (une vie et un peu de gras en moins) : le mobilier
  urbain (barrières de travaux, vélos, poubelles, sur lesquels on peut monter),
  les **voitures** (on peut sauter sur le toit) et les **motos** qui arrivent
  en face, et le **tram B**, annoncé par « DING DING », sur le toit duquel il faut sauter (pas
  de caténaire, c'est l'APS) sinon… crêpe.
- **Mines** (petit dôme kaki, voyant rouge qui clignote) : il faut sauter
  par-dessus. Sinon BOUM : Acaboulette part en morceaux, partie terminée.
- **Kebab** (rare, et un offert au départ) : gros plan sur Acaboulette qui
  tire la langue, « Un kebab ! Je deviens tarpin puissant !! », puis **KEBAB
  TURBO** : super vitesse et indestructible pendant 12 secondes.
- **Char d'assaut** (de temps en temps, à l'arrêt, signalé par une flèche
  verte « SAUTE DEDANS ! ») : il faut **sauter dedans**. Seule la tête
  d'Acaboulette dépasse de la tourelle, casque sur les oreilles et gros
  joint au bec. Le char ne roule pas vite mais il est **indestructible** :
  il écrase tout, mines comprises, et **pulvérise le tram B**.
  **Clic gauche** (ou Espace) : le canon tire un énorme **obus** tout droit,
  qui traverse tout. **Clic droit** (ou flèche bas) : une salve de
  **missiles à tête chercheuse** qui vont chercher obstacles, véhicules,
  mines et bestioles, mouettes comprises. Au bout de 14 secondes, plus de
  gazole : le char saute et Acaboulette repart à pattes.
- **Mode GAVÉ** : Acaboulette part tout mince et grossit en mangeant. Quand
  il devient énorme, pause litière (il pousse sa crotte en lisant le journal,
  « Je suis tarpin gavé… »), puis mode **GAVÉ** : invincible, il défonce tout,
  voitures, tram et mines compris. À la fin, tout est resté dans la litière :
  il redevient mince.

Les pauses gros plan attendent un clic (ou Espace) pour reprendre.

Le voyage passe par quatre régions, en boucle :

- **Bordeaux** : quai des Chartrons, place de la Bourse et Miroir d'eau,
  pont de pierre, Grosse Cloche, rue Sainte-Catherine (sous la pluie) et
  Grand-Théâtre, Cité du Vin, pont Chaban-Delmas ;
- **Marseille** (on y arrive au petit matin, après la première nuit) :
  Vieux-Port, Canebière, le Panier, Notre-Dame de la Garde, stade Vélodrome,
  corniche Kennedy et château d'If ;
- **Forcalquier** : place du Bourguet un jour de marché, la Citadelle, les
  champs de lavande, l'oliveraie, les Mourres ;
- **l'Algérie** : la Grande Poste d'Alger, la Casbah, Maqam Echahid,
  Notre-Dame d'Afrique, le Sahara.

Le décor s'adapte : façades, monuments, couleur de l'eau, ciel et messages.
Le tram B devient le **petit train des touristes** à Marseille, un
**tracteur et ses remorques de foin** à Forcalquier (pas de rails là-bas),
et le tram d'Alger en Algérie : même danger, même toit sur lequel sauter. Sur l'écran titre, on choisit sa **ville de départ** en cliquant
sur son bouton (ou flèches gauche / droite ou touches 1 à 4, puis Espace).
Après une partie, **V** revient à l'écran titre pour en changer. Le jour
tombe puis se lève à mesure qu'on avance.

Le record est enregistré dans `~/.local/share/packarch-acaboulette/`
(`%APPDATA%\packarch-acaboulette\` sous Windows).

## Compilation

```sh
cargo build --release
./target/release/packarch-acaboulette
```

Dépendances à l'exécution (chargées dynamiquement) : OpenGL (libglvnd),
X11 (libx11, libxi) ou Wayland (wayland, libxkbcommon).

### Windows

Compilation croisée depuis Linux (Arch : `mingw-w64-gcc` fournit l'éditeur
de liens) :

```sh
sudo pacman -S mingw-w64-gcc
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

Le résultat est `target/x86_64-pc-windows-gnu/release/packarch-acaboulette.exe`,
un exécutable autonome (aucune DLL à fournir, il suffit d'OpenGL).

Pour tout recompiler d'un coup (Linux et Windows), `./build.sh` place les
deux exécutables dans `dist/`. Avec `./build.sh --clear`, tout est nettoyé
(`target/` et `dist/`) avant de compiler.

Directement sous Windows, un simple `cargo build --release` suffit.

## Licence

GPL-3.0-or-later (voir `COPYING`).
