# AttackTower

POC d'un jeu de tower defense bilatéral en 3D, écrit en Rust avec Bevy et Avian3d.

## Stack technique

- **Langage** : Rust
- **Moteur** : Bevy (dernière version, choisie par cargo)
- **Physique** : Avian3d
- **Environnement de dev** : Nix flake (Vulkan, Wayland/X11, mold linker)

## Concept

Tower defense bilatéral en temps réel : un joueur à gauche, un joueur à droite. Chaque joueur achète des unités qui marchent en ligne droite vers la base adverse. Le premier à détruire la base adverse gagne.

Pour ce POC, les deux joueurs sont contrôlés depuis la même souris (test local).

## Spécifications

### Bases
- 1 base par joueur (gauche / droite)
- **20 PV** chacune
- Représentation : cube coloré

### Unités
- **10 PV**, **3 d'attaque**
- Coût : **1 or**
- Se déplacent tout droit vers la base adverse
- Représentation : cylindre coloré

### Combat
- **Mêlée entre unités** : deux unités ennemies qui se rencontrent s'arrêtent et s'attaquent jusqu'à la mort de l'une
- **Attaque de base** : au contact, l'unité inflige ses dégâts en boucle jusqu'à mourir ou détruire la base

### Économie
- **10 or** au départ pour chaque joueur
- **Revenu passif régulier** : +1 or toutes les X secondes pour chaque joueur

### Caméra & carte
- Carte horizontale, bases alignées sur l'axe gauche/droite
- Caméra **fixe** au milieu, en vue 3/4 à 45° du dessus
- Distance suffisante pour voir les deux bases simultanément
- Jeu prévu pour deux joueurs sur le même écran (pas de split-screen)

### UI
- **Deux boutons d'achat** d'unité (un par joueur) en bas de l'écran
- Compteur d'or visible à côté de chaque bouton

### Fin de partie
- Quand une base atteint 0 PV : texte **« Player X wins »** affiché
- **Bouton Restart** pour relancer une partie

### Direction artistique (POC)
- Formes géométriques primitives (cubes, cylindres)
- Couleurs unies (sol vert uni, couleurs distinctes par camp)
- Pas de textures ni de modèles importés

## Lancer le projet

```sh
# Dans le dev shell Nix (auto via direnv)
cargo run
```
