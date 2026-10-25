// SPDX-License-Identifier: GPL-3.0-or-later
// Evita a janela de console extra no Windows em release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    apuracao_rj_lib::run()
}
