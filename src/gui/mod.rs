// GUIモジュール: エディタUIをまとめる。
use nih_plug::prelude::Editor;
use std::sync::Arc;

use crate::params::BddPlusParams;

// エディタUIを描画するモジュール。
mod editor;
// UIテーマ定義。
mod theme;

/// プラグインのエディタUIを生成する。
///
/// # 引数
/// * `params` - BddPlusプラグインのパラメータ。
///
/// # 戻り値
/// エディタが生成できた場合はSome、失敗した場合はNone。
pub fn create_editor(params: Arc<BddPlusParams>) -> Option<Box<dyn Editor>> {
    // GUIエディタを生成する。
    editor::create_editor(params)
}
