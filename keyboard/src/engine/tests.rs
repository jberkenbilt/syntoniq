use super::*;
use crate::layout::RowCol;
use crate::test_util::TestController;

#[tokio::test]
async fn test_test_controller() -> anyhow::Result<()> {
    // Make sure basic test controller startup/shutdown works.
    let mut tc = TestController::new().await;
    let ts = tc.get_engine_state().await;
    assert!(ts.current_layout().is_none());
    let ws = tc.get_web_state().await;
    assert!(ws.selected_layout.is_empty());
    loop {
        let ws = tc.get_web_state().await;
        if ws.layout_names.is_empty() {
            continue;
        }
        break;
    }
    assert_eq!(ws.layout_names[0], "12-EDO-2x1");
    tc.shutdown().await
}

#[tokio::test]
async fn test_layout_selection() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select 12-EDO
    tc.press_and_release_key(KeyData::Layout { idx: 0 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    assert_eq!(ts.current_layout().unwrap().read().await.name, "12-EDO-2x1");
    let ws = tc.get_web_state().await;
    assert_eq!(ws.selected_layout, "12-EDO-2x1");
    assert_eq!(ws.base_pitch, "220*^1|4");
    tc.shutdown().await
}

#[tokio::test]
async fn test_sustain() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select 12-EDO
    tc.press_and_release_key(KeyData::Layout { idx: 0 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;

    // TODO: Launchpad-specific
    let middle_c_12_edo = KeyData::Other { position: 32 };

    // Press and release middle C. Note is on after press and off after release.
    tc.press_key(middle_c_12_edo).await?; // middle C
    tc.wait_for_test_event(TestEvent::HandledNote).await;
    let ts = tc.get_engine_state().await;
    let middle_c = Pitch::must_parse("220*^3|12");
    assert!(*ts.pitch_on_count.get(&middle_c).unwrap() > 0);
    assert_eq!(ts.last_note_for_pitch.get(&middle_c).unwrap().name, "C");
    tc.release_key(middle_c_12_edo).await?; // middle C
    tc.wait_for_test_event(TestEvent::HandledNote).await;
    let ts = tc.get_engine_state().await;
    assert!(!ts.pitch_on_count.contains_key(&middle_c));

    // Enter sustain mode
    tc.press_and_release_key(KeyData::Sustain).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.sustain);

    // Press and release middle C. Note stays on.
    tc.press_and_release_key(middle_c_12_edo).await?; // middle C
    tc.wait_for_test_event(TestEvent::HandledNote).await;
    tc.wait_for_test_event(TestEvent::HandledNote).await;
    let ts = tc.get_engine_state().await;
    assert!(*ts.pitch_on_count.get(&middle_c).unwrap() > 0);

    // Press and release middle C. Note turns off.
    tc.press_and_release_key(middle_c_12_edo).await?; // middle C
    tc.wait_for_test_event(TestEvent::HandledNote).await;
    tc.wait_for_test_event(TestEvent::HandledNote).await;
    let ts = tc.get_engine_state().await;
    assert!(!ts.pitch_on_count.contains_key(&middle_c));

    // Leave sustain mode
    tc.press_and_release_key(KeyData::Sustain).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(!ts.sustain);

    tc.shutdown().await
}

#[tokio::test]
async fn test_shift_key() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select a layout.
    tc.press_and_release_key(KeyData::Layout { idx: 1 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    // Press and release shift with no other notes.
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_key_state, ShiftKeyState::Off));
    tc.press_key(KeyData::Shift).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_key_state, ShiftKeyState::Down));
    tc.release_key(KeyData::Shift).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_key_state, ShiftKeyState::On));
    // Press and release shift again to toggle.
    tc.press_and_release_key(KeyData::Shift).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_key_state, ShiftKeyState::Off));
    // Press shift key without releasing
    tc.press_key(KeyData::Shift).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_key_state, ShiftKeyState::Down));
    // Press some other key
    tc.press_key(KeyData::Other { position: 11 }).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_key_state, ShiftKeyState::On));
    // Release other key
    tc.release_key(KeyData::Other { position: 11 }).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_key_state, ShiftKeyState::On));
    // Release shift key
    tc.release_key(KeyData::Shift).await?;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_key_state, ShiftKeyState::Off));
    Ok(())
}

#[tokio::test]
async fn test_transpose_cancels() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select 19-EDO
    tc.press_and_release_key(KeyData::Layout { idx: 1 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;

    // Enter transpose mode
    tc.press_and_release_key(KeyData::Transpose).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose_state, TransposeState::Pending { .. }));

    // Touch transpose to cancel
    tc.press_and_release_key(KeyData::Transpose).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose_state, TransposeState::Off));

    // Enter transpose mode
    tc.press_and_release_key(KeyData::Transpose).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose_state, TransposeState::Pending { .. }));
    // Touch a note
    tc.press_and_release_key(KeyData::Other { position: 32 })
        .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(
        ts.transpose_state,
        TransposeState::FirstSelected { .. }
    ));
    // Transpose key cancels
    tc.press_and_release_key(KeyData::Transpose).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose_state, TransposeState::Off));

    tc.shutdown().await
}

#[tokio::test]
async fn test_layout_shift_cancels() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;

    // Start shift
    tc.press_key(KeyData::Shift).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;

    // Touch a note
    tc.press_and_release_key(KeyData::Other { position: 32 })
        .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;

    // Release shift; cancels operation
    tc.release_key(KeyData::Shift).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;

    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_key_state, ShiftKeyState::Off));
    assert!(matches!(ts.shift_layout_state, ShiftLayoutState::Off));

    // Select generic layout
    tc.press_and_release_key(KeyData::Layout { idx: 4 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(
        ts.current_layout().unwrap().read().await.scale.scale_type,
        ScaleType::Generic(_)
    ));
    // Enter shift mode
    tc.press_key(KeyData::Shift).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    // Can't shift a generic layout
    tc.press_and_release_key(KeyData::Other { position: 32 })
        .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    tc.press_and_release_key(KeyData::Other { position: 33 })
        .await?;
    tc.wait_for_test_event(TestEvent::MoveCanceled).await;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_layout_state, ShiftLayoutState::Off));

    tc.shutdown().await
}

#[tokio::test]
async fn test_octave_transpose() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select 19-EDO
    tc.press_and_release_key(KeyData::Layout { idx: 1 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    assert_eq!(
        ts.current_layout()
            .unwrap()
            .read()
            .await
            .scale
            .base_pitch
            .to_string(),
        "220*^1|4"
    );

    // Go down an octave
    tc.press_and_release_key(KeyData::OctaveShift { up: false })
        .await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    assert_eq!(
        ts.current_layout()
            .unwrap()
            .read()
            .await
            .scale
            .base_pitch
            .to_string(),
        "110*^1|4"
    );

    // Go up two octaves
    tc.press_and_release_key(KeyData::OctaveShift { up: true })
        .await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    tc.press_and_release_key(KeyData::OctaveShift { up: true })
        .await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    assert_eq!(
        ts.current_layout()
            .unwrap()
            .read()
            .await
            .scale
            .base_pitch
            .to_string(),
        "440*^1|4"
    );

    tc.shutdown().await
}

#[tokio::test]
async fn test_transpose_same_layout() -> anyhow::Result<()> {
    // TODO: Launchpad-specific pitches
    let mut tc = TestController::new().await;
    // Select 19-EDO
    tc.press_and_release_key(KeyData::Layout { idx: 1 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    assert_eq!(
        ts.current_layout().unwrap().read().await.scale.name,
        "19-EDO"
    );

    // Enter move mode
    tc.press_and_release_key(KeyData::Transpose).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose_state, TransposeState::Pending { .. }));

    // Touch a note twice to transpose
    tc.press_and_release_key(KeyData::Other { position: 88 })
        .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(
        ts.transpose_state,
        TransposeState::FirstSelected { .. }
    ));
    // Touch a different note, changing first note
    tc.press_and_release_key(KeyData::Other { position: 44 })
        .await?;
    let ts = tc.get_engine_state().await;
    assert!(matches!(
        ts.transpose_state,
        TransposeState::FirstSelected { .. }
    ));
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    tc.press_and_release_key(KeyData::Other { position: 44 })
        .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose_state, TransposeState::Off));
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    // Transpose up 8 19-EDO steps: 1/4 + 8/19 = 51/76
    assert_eq!(
        ts.current_layout()
            .unwrap()
            .read()
            .await
            .scale
            .base_pitch
            .to_string(),
        "220*^51|76"
    );

    tc.shutdown().await
}

#[tokio::test]
async fn test_transpose_different_layout() -> anyhow::Result<()> {
    // TODO: Launchpad-specific pitches

    let mut tc = TestController::new().await;
    // Select 12-EDO
    tc.press_and_release_key(KeyData::Layout { idx: 0 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    assert_eq!(
        ts.current_layout().unwrap().read().await.scale.name,
        "12-EDO"
    );

    // Enter transpose mode
    tc.press_and_release_key(KeyData::Transpose).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose_state, TransposeState::Pending { .. }));

    // Select 19-EDO
    tc.press_and_release_key(KeyData::Layout { idx: 1 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    let edo_19 = ts.current_layout().clone().unwrap();
    assert_eq!(
        ts.current_layout().unwrap().read().await.scale.name,
        "19-EDO"
    );

    // Touch a note twice to transpose
    tc.press_and_release_key(KeyData::Other { position: 34 })
        .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    tc.press_and_release_key(KeyData::Other { position: 34 })
        .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose_state, TransposeState::Off));
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    // 19-EDO didn't change, but...
    {
        let layout = edo_19.read().await;
        assert_eq!(layout.scale.name, "19-EDO");
        assert_eq!(layout.scale.base_pitch.to_string(), "220*^1|4");
    }
    // 12-EDO did by 4 19-EDO steps. 1/4 + 6/19 = 43. Also, this is now the selected layout.
    {
        let lock = ts.current_layout();
        let layout = lock.as_ref().unwrap().read().await;
        assert_eq!(layout.scale.name, "12-EDO");
        assert_eq!(layout.scale.base_pitch.to_string(), "220*^43|76");
    }
    tc.shutdown().await
}

#[tokio::test]
async fn test_shift_layout() -> anyhow::Result<()> {
    // TODO: Launchpad-specific
    let mut tc = TestController::new().await;
    // Select 31-EDO
    tc.press_and_release_key(KeyData::Layout { idx: 2 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    {
        let lock = ts.current_layout();
        let layout = lock.as_ref().unwrap().read().await;
        assert_eq!(layout.scale.name, "31-EDO");
        assert_eq!(layout.base, Some(RowCol { row: 2, col: 2 }));
    }

    // Enter shift mode
    tc.press_key(KeyData::Shift).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    // Touch a note, then another to shift
    tc.press_and_release_key(KeyData::Other { position: 34 })
        .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(
        ts.shift_layout_state,
        ShiftLayoutState::FirstSelected { .. }
    ));
    // Over 1 column, up 2 rows
    tc.press_and_release_key(KeyData::Other { position: 55 })
        .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift_layout_state, ShiftLayoutState::Off));
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    {
        let lock = ts.current_layout();
        let layout = lock.as_ref().unwrap().read().await;
        assert_eq!(layout.scale.name, "31-EDO");
        assert_eq!(layout.base, Some(RowCol { row: 4, col: 3 }));
    }
    // Release shift
    tc.release_key(KeyData::Shift).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;

    tc.shutdown().await
}

#[tokio::test]
async fn transpose_non_note_to_note() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Press any key on the start screen.
    tc.press_key(KeyData::Other { position: 55 }).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.is_empty());
    // Select a layout
    tc.press_and_release_key(KeyData::Layout { idx: 0 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    // Still not seen as down.
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.is_empty());
    // Release the key
    tc.release_key(KeyData::Other { position: 55 }).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.is_empty());

    tc.shutdown().await
}

#[tokio::test]
async fn transpose_note_to_non_note() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select a layout
    tc.press_and_release_key(KeyData::Layout { idx: 0 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    // Press a note key
    tc.press_key(KeyData::Other { position: 18 }).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    // Observe that the key is down and the pitch is playing.
    assert!(ts.positions_down.contains_key(&18));
    let layout_101_pos_18 = ts.notes.get(&18).unwrap().as_ref().unwrap().pitch.clone();
    assert_eq!(ts.pitch_on_count.get(&layout_101_pos_18).unwrap_or(&0), &1);
    // Select a different layout that has the note
    tc.press_and_release_key(KeyData::Layout { idx: 1 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    // The key is still down, and the pitch has changed.
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.contains_key(&18));
    let layout_102_pos_18 = ts.notes.get(&18).unwrap().as_ref().unwrap().pitch.clone();
    assert_ne!(layout_101_pos_18, layout_102_pos_18);
    assert_eq!(ts.pitch_on_count.get(&layout_101_pos_18).unwrap_or(&0), &0);
    assert_eq!(ts.pitch_on_count.get(&layout_102_pos_18).unwrap_or(&0), &1);
    // Select a layout that doesn't have the note
    tc.press_and_release_key(KeyData::Layout { idx: 4 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.is_empty());
    assert_eq!(ts.pitch_on_count.get(&layout_101_pos_18).unwrap_or(&0), &0);
    assert_eq!(ts.pitch_on_count.get(&layout_102_pos_18).unwrap_or(&0), &0);
    tc.release_key(KeyData::Other { position: 18 }).await?;

    tc.shutdown().await
}

#[tokio::test]
async fn transpose_with_sustain() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select a layout
    tc.press_and_release_key(KeyData::Layout { idx: 0 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    // Enter sustain
    tc.press_and_release_key(KeyData::Sustain).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    // Press a note key
    tc.press_key(KeyData::Other { position: 18 }).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    // Observe that the key is down and the pitch is playing.
    assert!(ts.positions_down.contains_key(&18));
    let pos_18a = ts.notes.get(&18).unwrap().as_ref().unwrap().pitch.clone();
    assert_eq!(ts.pitch_on_count.get(&pos_18a).unwrap_or(&0), &1);
    // Select a different layout that has the note by shifting down an octave
    tc.press_and_release_key(KeyData::OctaveShift { up: false })
        .await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    // The key is still down, and the new and old pitches are both playing.
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.contains_key(&18));
    let pos_18b = ts.notes.get(&18).unwrap().as_ref().unwrap().pitch.clone();
    assert_ne!(pos_18a, pos_18b);
    assert_eq!(ts.pitch_on_count.get(&pos_18a).unwrap_or(&0), &1);
    assert_eq!(ts.pitch_on_count.get(&pos_18b).unwrap_or(&0), &1);
    // Release the key. The notes are still both on.
    tc.release_key(KeyData::Other { position: 18 }).await?;
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.is_empty());
    assert_eq!(ts.pitch_on_count.get(&pos_18a).unwrap_or(&0), &1);
    assert_eq!(ts.pitch_on_count.get(&pos_18b).unwrap_or(&0), &1);

    tc.shutdown().await
}

#[tokio::test]
async fn transpose_print_notes() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select Just Intonation
    tc.press_and_release_key(KeyData::Layout { idx: 4 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    // Enter sustain
    tc.press_and_release_key(KeyData::Sustain).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    // Play some notes
    tc.press_and_release_key(KeyData::Other { position: 51 })
        .await?;
    tc.press_and_release_key(KeyData::Other { position: 53 })
        .await?;
    tc.press_and_release_key(KeyData::Other { position: 55 })
        .await?;
    // Transpose and play
    tc.press_and_release_key(KeyData::Transpose).await?;
    // Since this is sustain, we have to turn the note on ahead so it won't be on after selecting
    tc.press_and_release_key(KeyData::Other { position: 13 })
        .await?;
    tc.press_and_release_key(KeyData::Other { position: 13 })
        .await?;
    tc.press_and_release_key(KeyData::Other { position: 13 })
        .await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    // More notes
    tc.press_and_release_key(KeyData::Other { position: 53 })
        .await?;
    tc.press_and_release_key(KeyData::Other { position: 55 })
        .await?;
    // New layout
    tc.press_and_release_key(KeyData::Layout { idx: 0 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    tc.press_and_release_key(KeyData::Other { position: 33 })
        .await?;
    // Let all events be handled
    tc.sync().await?;
    let ts = tc.get_engine_state().await;
    let exp: Vec<String> = [
        "Scale: 12-EDO, base=220*^1|4",
        "  Note: D (0.2), pitch=220*^5|12 (220*^1|4 × ^1|6)",
        "Scale: JI-11, base=55*^1|4",
        "  Note: C ([31]*2), pitch=220*^1|4 (55*^1|4 × 4)",
        "  Note: E ([33]*2), pitch=275*^1|4 (55*^1|4 × 5)",
        "  Note: G ([35]*2), pitch=330*^1|4 (55*^1|4 × 6)",
        "Scale: JI-11, base=68.75*^1|4 (transposition: 55*^1|4 × 5/4)",
        "  Note: E ([33]*2), pitch=343.75*^1|4 (68.75*^1|4 × 5)",
        "  Note: G ([35]*2), pitch=412.5*^1|4 (68.75*^1|4 × 6)",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert_eq!(ts.current_played_notes(), exp);

    tc.shutdown().await
}

// #[tokio::test]
// async fn test_scroll_layouts() -> anyhow::Result<()> {
//     // TODO: launchpad-specific test
//     let mut tc = TestController::new().await;
//
//     tc.press_and_release_key(KeyData::Layout { idx: 4 }).await?;
//     tc.wait_for_test_event(TestEvent::LayoutSelected).await;
//     let ts = tc.get_engine_state().await;
//     assert_eq!(ts.layout.unwrap(), 4);
//     // Scroll layout
//     tc.press_and_release_key(KeyData::Other { position: 19 }).await?;
//     tc.wait_for_test_event(TestEvent::LayoutsScrolled).await;
//     let ts = tc.get_engine_state().await;
//     assert_eq!(ts.layout_offset, 8);
//     tc.press_and_release_key(KeyData::Layout { idx: 0 }).await?;
//     tc.wait_for_test_event(TestEvent::LayoutSelected).await;
//     let ts = tc.get_engine_state().await;
//     assert_eq!(ts.layout.unwrap(), 8);
//
//     tc.press_and_release_key(KeyData::Other { position: 19 }).await?;
//     tc.wait_for_test_event(TestEvent::LayoutsScrolled).await;
//     let ts = tc.get_engine_state().await;
//     assert_eq!(ts.layout_offset, 0);
//     tc.press_and_release_key(KeyData::Layout { idx: 1 }).await?;
//     tc.wait_for_test_event(TestEvent::LayoutSelected).await;
//     let ts = tc.get_engine_state().await;
//     assert_eq!(ts.layout.unwrap(), 1);
//
//     tc.shutdown().await
// }
