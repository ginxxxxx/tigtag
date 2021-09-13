use super::*;

mod map;
mod player;
mod chasers;
mod countdown_ui;
mod util;

pub use map::*;
pub use player::*;
pub use chasers::*;
pub use countdown_ui::*;
pub use util::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

//Pluginの手続き
pub struct PluginGamePlay;
impl Plugin for PluginGamePlay
{	fn build( &self, app: &mut AppBuilder )
	{	app
		//------------------------------------------------------------------------------------------
		.init_resource::<Record>()										// スコア等のリソース
		.init_resource::<MapInfo>()										// マップ情報のリソース
		//------------------------------------------------------------------------------------------
		.add_system_set													// ＜GameState::GameStart＞
		(	SystemSet::on_enter( GameState::GameStart )					// ＜on_enter()＞
				.with_system( show_message_start.system() )				// スタートメッセージを表示する
				.with_system( reset_gamestart_counter.system() )		// カウントダウン用のカウンタークリア
		)
		.add_system_set													// ＜GameState::GameStart＞
		(	SystemSet::on_enter( GameState::GameStart )					// ＜on_enter()＞
				.label( Label::GenerateMap )							// ＜label＞
				.with_system( spawn_sprite_new_map.system() )			// 新マップを生成して表示
		)
		.add_system_set													// ＜GameState::GameStart＞
		(	SystemSet::on_enter( GameState::GameStart )					// ＜on_enter()＞
				.after( Label::GenerateMap )							// ＜after＞
				.with_system( spawn_sprite_player.system() )			// 自機を配置(マップ生成後)
				.with_system( spawn_sprite_chasers.system() )			// 追手を配置
		)
		.add_system_set													// ＜GameState::GameStart＞
		(	SystemSet::on_update( GameState::GameStart )				// ＜on_update()＞
				.with_system( change_state_gameplay_with_cd.system() )	// カウントダウン終了⇒GamePlayへ遷移
		)
		.add_system_set													// ＜GameState::GameStart＞
		(	SystemSet::on_exit( GameState::GameStart )					// ＜on_exit()＞
				.with_system( hide_message_start.system() )				// スタートメッセージを隠す
		)
		//------------------------------------------------------------------------------------------
		.add_system_set													// ＜GameState::GamePlay＞
		(	SystemSet::on_update( GameState::GamePlay )					// ＜on_update()＞
				.before( Label::MoveSpriteCharacters )					// ＜before＞
				.with_system( detect_score_and_collision.system() )		// クリア⇒GameClear、衝突⇒GameOver
		)
		.add_system_set													// ＜GameState::GamePlay＞
		(	SystemSet::on_update( GameState::GamePlay )					// ＜on_update()＞
				.label( Label::MoveSpriteCharacters )					// ＜label＞
				.with_system( move_sprite_player.system() )				// 自機のスプライトを移動する
				.with_system( move_sprite_chaser.system() )				// 追手のスプライトを移動する
		)
		//------------------------------------------------------------------------------------------
		.add_system_set													// ＜GameState::GameClear＞
		(	SystemSet::on_enter( GameState::GameClear )					// ＜on_enter()＞
				.with_system( show_message_clear.system() )				// クリアメッセージを表示する
				.with_system( reset_gameclear_counter.system() )		// カウントダウン用のカウンタークリア
		)
		.add_system_set													// ＜GameState::GameClear＞
		(	SystemSet::on_update( GameState::GameClear )				// ＜on_update()＞
				.with_system( change_state_gamestart_with_cd.system() )	// カウントダウン終了⇒GameStartへ遷移
		)
		.add_system_set													// ＜GameState::Clear＞
		(	SystemSet::on_exit( GameState::GameClear )					// ＜on_exit()＞
				.with_system( hide_message_clear.system() )				// クリアメッセージを隠す
				.with_system( increment_record.system() )				// ステージを＋１する
		)
		//------------------------------------------------------------------------------------------
		.add_system_set													// ＜GameState::GameOver＞
		(	SystemSet::on_enter( GameState::GameOver )					// ＜on_enter()＞
				.with_system( show_message_over.system() )				// ゲームオーバーを表示する
				.with_system( reset_gameover_counter.system() )			// カウントダウン用のカウンタークリア
		)
		.add_system_set													// ＜GameState::GameOver＞
		(	SystemSet::on_update( GameState::GameOver )					// ＜on_update()＞
				.with_system( change_state_gamestart_by_key.system() )	// SPACEキー入力⇒GameStartへ遷移
				.with_system( change_state_demostart_with_cd.system() )	// カウントダウン終了⇒DemoStartへ遷移
		)
		.add_system_set													// ＜GameState::GameOver＞
		(	SystemSet::on_exit( GameState::GameOver )					// ＜on_exit()＞
				.with_system( hide_message_over.system() )				// ゲームオーバーを隠す
				.with_system( clear_record.system() )					// スコアとステージを初期化
		)
		//------------------------------------------------------------------------------------------
		;
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

//得点と衝突を判定する。クリアならGameClearへ、衝突ならGameOverへ遷移する
pub fn detect_score_and_collision
(	q_player  : Query<&Player>,
	q_chaser  : Query<&Chaser>,
	mut state : ResMut<State<GameState>>,
	mut record: ResMut<Record>,
	mut map   : ResMut<MapInfo>,
	mut cmds  : Commands,
)
{	let is_demoplay = matches!( state.current(), GameState::DemoPlay );

	//自機のgrid座標のオブジェクトがドットなら
	let player = q_player.single().unwrap();
	let ( p_grid_x, p_grid_y ) = player.grid_position;
	if let MapObj::Dot( opt_dot ) = map.array[ p_grid_x ][ p_grid_y ]
	{	//得点処理
		record.score += 1;
		map.array[ p_grid_x ][ p_grid_y ] = MapObj::Space;
		map.count_dots -= 1;
		cmds.entity( opt_dot.unwrap() ).despawn();

		//ハイスコアの更新
		if ! is_demoplay && record.score > record.high_score
		{	record.high_score = record.score;
		}

		//クリアならeventをセットして関数から脱出
		if map.count_dots == 0
		{	let next = if is_demoplay { GameState::DemoLoop } else { GameState::GameClear };
			let _ = state.overwrite_set( next );
			return;
		}
	}

	//追手と自機のpixel座標が衝突しているか？
	let ( mut p_new_x, mut p_new_y ) = player.pixel_position;
	let ( mut p_old_x, mut p_old_y ) = player.pixel_position_old;
	for chaser in q_chaser.iter()
	{	let ( c_grid_x, c_grid_y ) = chaser.grid_position;
		let ( mut c_new_x, mut c_new_y ) = chaser.pixel_position;
		let ( mut c_old_x, mut c_old_y ) = chaser.pixel_position_old;

		let is_collision =
			if p_grid_y == c_grid_y			//Y軸が一致するなら
			{	if p_new_x > p_old_x { std::mem::swap( &mut p_new_x, &mut p_old_x ) }
				if c_new_x > c_old_x { std::mem::swap( &mut c_new_x, &mut c_old_x ) }

				( p_new_x..=p_old_x ).contains( &c_new_x ) ||
				( p_new_x..=p_old_x ).contains( &c_old_x ) ||
				( c_new_x..=c_old_x ).contains( &p_new_x ) ||	
				( c_new_x..=c_old_x ).contains( &p_old_x )
			}
			else if p_grid_x == c_grid_x 	//X軸が一致するなら
			{	if p_new_y > p_old_y { std::mem::swap( &mut p_new_y, &mut p_old_y ) }
				if c_new_y > c_old_y { std::mem::swap( &mut c_new_y, &mut c_old_y ) }

				( p_new_y..=p_old_y ).contains( &c_new_y ) ||
				( p_new_y..=p_old_y ).contains( &c_old_y ) ||
				( c_new_y..=c_old_y ).contains( &p_new_y ) ||
				( c_new_y..=c_old_y ).contains( &p_old_y )
			}
			else
			{ false };

		//衝突ならeventをセットして関数から脱出
		if is_collision
		{	let next = if is_demoplay { GameState::DemoLoop } else { GameState::GameOver };
			let _ = state.overwrite_set( next );
			return;
		}
	}
}

//SPACEキーが入力されたらGameStartへ遷移する
pub fn change_state_gamestart_by_key
(	mut inkey: ResMut<Input<KeyCode>>,
	mut state: ResMut<State<GameState>>,
)
{	if inkey.just_pressed( KeyCode::Space ) 
	{	let _ = state.overwrite_set( GameState::GameStart );

		//https://bevy-cheatbook.github.io/programming/states.html#with-input
		inkey.reset( KeyCode::Space );
	}
}

//End of code.