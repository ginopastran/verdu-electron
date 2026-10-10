mod comun;

use andexmarket::app::Ruta;

#[test]
fn arranque() {
    comun::capturar("00-arranque", comun::global(Ruta::AdminLogin), |_| {});
}
