# Acaboulette

Un petit jeu d'arcade (gore désactivable) pour Packarch, écrit en Rust avec
[macroquad](https://macroquad.rs), dans l'esprit de *Paf le Bengal*.

**Acaboulette**, un gros matou roux, traverse Bordeaux au pas de course et
mange ou écrabouille tout ce qui passe : les pigeons de la Bourse, les rats
de la Garonne, les chiens… C'est tarpin gore.

Tout est dessiné par le code : aucune image ni aucun son à charger.

## Commandes

- **Espace / clic / flèche haut** : sauter (double saut ; maintenir = plus haut)
- **Flèche bas / S / clic droit** (en l'air) : **plat ventre**, l'onde de choc
  écrabouille tout autour
- **G** : activer / désactiver le gore
- **Échap** : quitter

## Le jeu

- **Pigeons, rats, chiens** : aucun danger, c'est à manger ! En courant
  dedans il les avale, en tombant dessus il les écrabouille. Plus il en
  mange, plus ça rapporte et plus il **grossit** (plats ventre plus larges,
  points multipliés).
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
- **Mode GAVÉ** : Acaboulette part tout mince et grossit en mangeant. Quand
  il devient énorme, pause litière (il pousse sa crotte en lisant le journal,
  « Je suis tarpin gavé… »), puis mode **GAVÉ** : invincible, il défonce tout,
  voitures, tram et mines compris. À la fin, tout est resté dans la litière :
  il redevient mince.

Les pauses gros plan attendent un clic (ou Espace) pour reprendre.

Les quartiers défilent en boucle : quai des Chartrons, place de la Bourse et
Miroir d'eau, pont de pierre, Grosse Cloche, rue Sainte-Catherine (sous la
pluie) et Grand-Théâtre, Cité du Vin, pont Chaban-Delmas. Le jour tombe puis
se lève à mesure qu'on avance.

Le record est enregistré dans `~/.local/share/packarch-acaboulette/`.

## Compilation

```sh
cargo build --release
./target/release/packarch-acaboulette
```

Dépendances à l'exécution (chargées dynamiquement) : OpenGL (libglvnd),
X11 (libx11, libxi) ou Wayland (wayland, libxkbcommon).

## Licence

GPL-3.0-or-later (voir `COPYING`).
