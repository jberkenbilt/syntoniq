use super::*;
use crate::test_util::TestController;
use syntoniq_common::pitch::Pitch;

fn pos(row: i32, col: i32) -> Coordinate {
    Coordinate { row, col }
}

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
    assert_eq!(ts.current_layout().unwrap().name, "12-EDO-2x1");
    let ws = tc.get_web_state().await;
    assert_eq!(ws.selected_layout, "12-EDO-2x1");
    tc.shutdown().await
}

#[tokio::test]
async fn test_sustain() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select 12-EDO
    tc.press_and_release_key(KeyData::Layout { idx: 0 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let middle_c_12_edo = KeyData::Note {
        position: pos(3, 2),
    };

    // Press and release middle C. Note is on after press and off after release.
    tc.press_key(middle_c_12_edo).await?; // middle C
    tc.wait_for_test_event(TestEvent::HandledNote).await;
    let ts = tc.get_engine_state().await;
    let middle_c = Pitch::must_parse("220*^3|12");
    assert!(*ts.pitch_on_count.get(&middle_c).unwrap() > 0);
    assert_eq!(
        ts.last_note_for_pitch.get(&middle_c).unwrap().placed.name,
        "c"
    );
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
async fn test_shift_and_transpose_keys() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select a layout.
    tc.press_and_release_key(KeyData::Layout { idx: 1 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;

    // Start a shift operation
    let ts = tc.get_engine_state().await;
    assert!(ts.shift.is_none());
    let ts = tc.shift().await?;
    assert!(matches!(ts.shift, Some(None)));

    // Press and release shift again to toggle.
    let ts = tc.shift().await?;
    assert!(ts.shift.is_none());

    // Start a shift operation
    let ts = tc.get_engine_state().await;
    assert!(ts.shift.is_none());
    let ts = tc.shift().await?;
    assert!(matches!(ts.shift, Some(None)));

    // A layout key cancels
    tc.press_and_release_key(KeyData::Layout { idx: 1 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.shift.is_none());

    // Press and release shift again to toggle.
    let ts = tc.shift().await?;
    assert!(matches!(ts.shift, Some(None)));

    // Prelease and release a key; shift is pending
    tc.press_and_release_key(KeyData::Note {
        position: pos(3, 4),
    })
    .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift, Some(Some(_))));

    // Press and release transpose; cancels shift and activates transpose.
    let ts = tc.transpose().await?;
    assert!(ts.shift.is_none());
    assert!(matches!(ts.transpose, Some(None)));

    // Prelease and release a key; transpose is pending
    tc.press_and_release_key(KeyData::Note {
        position: pos(3, 4),
    })
    .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose, Some(Some(_))));

    // Press and release transpose without finishing; cancels
    let ts = tc.transpose().await?;
    assert!(ts.transpose.is_none());

    // You can also enter with a key press alone.
    let ts = tc.get_engine_state().await;
    assert!(ts.shift.is_none());
    tc.press_key(KeyData::Shift).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift, Some(None)));

    tc.press_and_release_key(KeyData::Note {
        position: pos(3, 4),
    })
    .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift, Some(Some(_))));

    tc.press_and_release_key(KeyData::Note {
        position: pos(4, 4),
    })
    .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.shift.is_none());

    let ts = tc.get_engine_state().await;
    assert!(ts.shift.is_none());
    tc.release_key(KeyData::Shift).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.shift.is_none());

    // Also works with transpose.
    let ts = tc.get_engine_state().await;
    assert!(ts.transpose.is_none());
    tc.press_key(KeyData::Transpose).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose, Some(None)));

    tc.press_and_release_key(KeyData::Note {
        position: pos(3, 4),
    })
    .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose, Some(Some(_))));

    tc.press_and_release_key(KeyData::Note {
        position: pos(4, 4),
    })
    .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.transpose.is_none());

    let ts = tc.get_engine_state().await;
    assert!(ts.transpose.is_none());
    tc.release_key(KeyData::Transpose).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.transpose.is_none());

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
        ts.current_layout().unwrap().mappings[0]
            .offsets
            .read()
            .expect("lock")
            .transpose
            .to_string(),
        "1"
    );

    // Go down an octave
    tc.press_and_release_key(KeyData::OctaveShift { up: false })
        .await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    assert_eq!(
        ts.current_layout().unwrap().mappings[0]
            .offsets
            .read()
            .expect("lock")
            .transpose
            .to_string(),
        "1/2"
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
        ts.current_layout().unwrap().mappings[0]
            .offsets
            .read()
            .expect("lock")
            .transpose
            .to_string(),
        "2"
    );

    tc.shutdown().await
}

#[tokio::test]
async fn test_transpose() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select 19-EDO
    tc.press_and_release_key(KeyData::Layout { idx: 1 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    assert_eq!(ts.current_layout().unwrap().name, "19-EDO-2x1");

    // Enter transpose mode
    let ts = tc.transpose().await?;
    assert!(matches!(ts.transpose, Some(None)));

    // Select the first note
    tc.press_and_release_key(KeyData::Note {
        position: pos(4, 4),
    })
    .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose, Some(Some(_))));
    assert_eq!(
        ts.current_layout().unwrap().mappings[0]
            .offsets
            .read()
            .expect("lock")
            .transpose,
        Pitch::unit(),
    );

    // Touch the target note. Up 1 and right 1 raises us 3 steps, so moving the current pitch
    // inverts that for the transposition.
    let layout1 = ts.current_layout().unwrap();
    assert_eq!(
        layout1.note_at_location(pos(4, 4)).unwrap().pitch,
        Pitch::must_parse("220*^1|4*^5|19")
    );
    tc.press_and_release_key(KeyData::Note {
        position: pos(5, 5),
    })
    .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.transpose.is_none());
    // Now the second note has the pitch previously belonging to the first note. Don't refetch
    // the layout. It's an Arc, so it should be updated in place.
    assert_eq!(
        layout1.note_at_location(pos(5, 5)).unwrap().pitch,
        Pitch::must_parse("220*^1|4*^5|19")
    );
    assert_eq!(
        layout1.mappings[0].offsets.read().expect("lock").transpose,
        Pitch::must_parse("^-3|19"),
    );

    // Assign that pitch to a note in a different layout. Enter transpose mode
    let ts = tc.transpose().await?;
    assert!(matches!(ts.transpose, Some(None)));

    // Select the first note
    tc.press_and_release_key(KeyData::Note {
        position: pos(5, 5),
    })
    .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.transpose, Some(Some(_))));

    // Select 31-EDO
    tc.press_and_release_key(KeyData::Layout { idx: 2 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    let layout2 = ts.current_layout().unwrap();
    assert_eq!(layout2.name, "31-EDO-5x3");
    assert_eq!(
        layout2.mappings[0].offsets.read().expect("lock").transpose,
        Pitch::unit(),
    );

    // Touch the target note.
    assert_eq!(
        layout2.note_at_location(pos(2, 3)).unwrap().pitch,
        Pitch::must_parse("220*^1|4*^5|31")
    );
    tc.press_and_release_key(KeyData::Note {
        position: pos(2, 3),
    })
    .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.transpose.is_none());
    // The first layout is unchanged.
    assert_eq!(
        layout1.mappings[0].offsets.read().expect("lock").transpose,
        Pitch::must_parse("^-3|19"),
    );
    // The second note has the pitch previously belonging to the first note.
    assert_eq!(
        layout2.note_at_location(pos(2, 3)).unwrap().pitch,
        Pitch::must_parse("220*^1|4*^5|19")
    );
    // The second mapping has been transposed by the difference.
    assert_eq!(
        layout2.mappings[0].offsets.read().expect("lock").transpose,
        Pitch::must_parse("^-5|31*^5|19"),
    );

    // Transpose within a single layout that has two mappings.
    tc.press_and_release_key(KeyData::Layout { idx: 5 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    let layout = ts.current_layout().unwrap();
    assert_eq!(layout.name, "JI-11+31-EDO");
    // Enter transpose mode
    let ts = tc.transpose().await?;
    assert!(matches!(ts.transpose, Some(None)));
    // Select the first note from the 31-EDO overlay
    tc.press_and_release_key(KeyData::Note {
        position: pos(7, 7),
    })
    .await?;
    tc.press_and_release_key(KeyData::Note {
        position: pos(5, 1),
    })
    .await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    // The first mapping is untouched. The second one is transposed.
    assert_eq!(
        layout.mappings[0].offsets.read().expect("lock").transpose,
        Pitch::unit()
    );
    assert_eq!(
        layout.mappings[1].offsets.read().expect("lock").transpose,
        // 264*^5|31 / 220^3|4
        Pitch::must_parse("264*^5|31*1/220*^-1|4"),
    );

    tc.shutdown().await
}

#[tokio::test]
async fn test_shift() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select 31-EDO
    tc.press_and_release_key(KeyData::Layout { idx: 2 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    let layout = ts.current_layout().unwrap();
    assert_eq!(layout.name, "31-EDO-5x3");
    {
        let offsets = layout.mappings[0].offsets.read().expect("lock");
        assert_eq!(offsets.shift_h, 0);
        assert_eq!(offsets.shift_v, 0);
    }
    // Enter shift mode
    tc.press_and_release_key(KeyData::Shift).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    // Touch a note, then another to shift
    tc.press_and_release_key(KeyData::Note {
        position: pos(3, 4),
    })
    .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(matches!(ts.shift, Some(Some(_))));
    // Over 1 column, up 2 rows
    tc.press_and_release_key(KeyData::Note {
        position: pos(5, 5),
    })
    .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.shift.is_none());
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    let layout = ts.current_layout().unwrap();
    {
        let offsets = layout.mappings[0].offsets.read().expect("lock");
        assert_eq!(offsets.shift_h, 1);
        assert_eq!(offsets.shift_v, 2);
    }

    // Shift within a single layout that has two mappings fails.
    tc.press_and_release_key(KeyData::Layout { idx: 5 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    let ts = tc.get_engine_state().await;
    let layout = ts.current_layout().unwrap();
    assert_eq!(layout.name, "JI-11+31-EDO");
    // Enter transpose mode
    let ts = tc.shift().await?;
    assert!(matches!(ts.shift, Some(None)));
    // Select the first note from the 31-EDO overlay
    tc.press_and_release_key(KeyData::Note {
        position: pos(7, 7),
    })
    .await?;
    tc.press_and_release_key(KeyData::Note {
        position: pos(5, 1),
    })
    .await?;
    tc.wait_for_test_event(TestEvent::MoveCanceled).await;

    tc.shutdown().await
}

#[tokio::test]
async fn layout_while_pressed() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Press any key on the start screen.
    tc.press_key(KeyData::Note {
        position: pos(5, 5),
    })
    .await?;
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
    tc.release_key(KeyData::Note {
        position: pos(5, 5),
    })
    .await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.is_empty());

    tc.shutdown().await
}

#[tokio::test]
async fn select_layout_while_playing() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select a layout
    tc.press_and_release_key(KeyData::Layout { idx: 0 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    // Press a note key
    let position = pos(1, 8);
    tc.press_key(KeyData::Note { position }).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    // Observe that the key is down and the pitch is playing.
    assert!(ts.positions_down.contains_key(&position));
    let layout_0_pos_18 = ts
        .notes
        .get(&position)
        .unwrap()
        .as_ref()
        .unwrap()
        .placed
        .pitch
        .clone();
    assert_eq!(ts.pitch_on_count.get(&layout_0_pos_18).unwrap_or(&0), &1);
    // Select a different layout that has the note
    tc.press_and_release_key(KeyData::Layout { idx: 1 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    // The key is still down, and the pitch has changed.
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.contains_key(&position));
    let layout_1_pos_18 = ts
        .notes
        .get(&position)
        .unwrap()
        .as_ref()
        .unwrap()
        .placed
        .pitch
        .clone();
    assert_ne!(layout_0_pos_18, layout_1_pos_18);
    assert_eq!(ts.pitch_on_count.get(&layout_0_pos_18).unwrap_or(&0), &0);
    assert_eq!(ts.pitch_on_count.get(&layout_1_pos_18).unwrap_or(&0), &1);
    // Select a layout that doesn't have the note
    tc.press_and_release_key(KeyData::Layout { idx: 4 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.is_empty());
    assert_eq!(ts.pitch_on_count.get(&layout_0_pos_18).unwrap_or(&0), &0);
    assert_eq!(ts.pitch_on_count.get(&layout_1_pos_18).unwrap_or(&0), &0);
    tc.release_key(KeyData::Note { position }).await?;

    tc.shutdown().await
}

#[tokio::test]
async fn octave_shift_with_sustain() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select a layout
    tc.press_and_release_key(KeyData::Layout { idx: 0 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    // Enter sustain
    tc.press_and_release_key(KeyData::Sustain).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    // Press a note key
    let position = pos(1, 8);
    tc.press_key(KeyData::Note { position }).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    let ts = tc.get_engine_state().await;
    // Observe that the key is down and the pitch is playing.
    assert!(ts.positions_down.contains_key(&position));
    let pos_18a = ts
        .notes
        .get(&position)
        .unwrap()
        .as_ref()
        .unwrap()
        .placed
        .pitch
        .clone();
    assert_eq!(ts.pitch_on_count.get(&pos_18a).unwrap_or(&0), &1);
    // Select a different layout that has the note by shifting down an octave
    tc.press_and_release_key(KeyData::OctaveShift { up: false })
        .await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    // The key is still down, and the new and old pitches are both playing.
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.contains_key(&position));
    let pos_18b = ts
        .notes
        .get(&position)
        .unwrap()
        .as_ref()
        .unwrap()
        .placed
        .pitch
        .clone();
    assert_ne!(pos_18a, pos_18b);
    assert_eq!(ts.pitch_on_count.get(&pos_18a).unwrap_or(&0), &1);
    assert_eq!(ts.pitch_on_count.get(&pos_18b).unwrap_or(&0), &1);
    // Release the key. The notes are still both on.
    tc.release_key(KeyData::Note { position }).await?;
    let ts = tc.get_engine_state().await;
    assert!(ts.positions_down.is_empty());
    assert_eq!(ts.pitch_on_count.get(&pos_18a).unwrap_or(&0), &1);
    assert_eq!(ts.pitch_on_count.get(&pos_18b).unwrap_or(&0), &1);

    tc.shutdown().await
}

#[tokio::test]
async fn print_notes() -> anyhow::Result<()> {
    let mut tc = TestController::new().await;
    // Select Just Intonation
    tc.press_and_release_key(KeyData::Layout { idx: 4 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    // Enter sustain
    tc.press_and_release_key(KeyData::Sustain).await?;
    tc.wait_for_test_event(TestEvent::HandledKey).await;
    // Play some notes
    tc.press_and_release_key(KeyData::Note {
        position: pos(5, 1),
    })
    .await?;
    tc.press_and_release_key(KeyData::Note {
        position: pos(5, 3),
    })
    .await?;
    tc.press_and_release_key(KeyData::Note {
        position: pos(5, 5),
    })
    .await?;
    // Transpose and play from the same layout/mapping. Old pitches are still there.
    tc.press_and_release_key(KeyData::Transpose).await?;
    tc.press_and_release_key(KeyData::Note {
        position: pos(1, 3),
    })
    .await?;
    tc.press_and_release_key(KeyData::Note {
        position: pos(1, 1),
    })
    .await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    // More notes
    tc.press_and_release_key(KeyData::Note {
        position: pos(5, 3),
    })
    .await?;
    tc.press_and_release_key(KeyData::Note {
        position: pos(5, 5),
    })
    .await?;
    // New layout
    tc.press_and_release_key(KeyData::Layout { idx: 0 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    tc.press_and_release_key(KeyData::Note {
        position: pos(3, 3),
    })
    .await?;
    // Layout with two mappings
    tc.press_and_release_key(KeyData::Layout { idx: 5 }).await?;
    tc.wait_for_test_event(TestEvent::LayoutSelected).await;
    tc.press_and_release_key(KeyData::Note {
        position: pos(2, 1),
    })
    .await?;
    tc.press_and_release_key(KeyData::Note {
        position: pos(7, 7),
    })
    .await?;
    // Let all events be handled
    tc.sync().await?;
    let ts = tc.get_engine_state().await;
    let exp: Vec<String> = [
        "Scale: 31-EDO, base=264",
        "  Note: d (pitch=264*^5|31, interval=^5|31)",
        "Scale: JI-11, base=220*^1|4",
        "  Note: c# (pitch=935/16*^1|4, interval=17/16)",
        "  Note: c (pitch=220*^1|4, interval=1)",
        "  Note: e (pitch=275*^1|4, interval=5/4)",
        "  Note: g (pitch=330*^1|4, interval=3/2)",
        "Scale: JI-11, base=275*^1|4 (transposition: 220*^1|4 × 5/4)",
        "  Note: e (pitch=343.75*^1|4, interval=5/4)",
        "  Note: g (pitch=412.5*^1|4, interval=3/2)",
        "Scale: default, base=220*^1|4",
        "  Note: d (pitch=220*^5|12, interval=^1|6)",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert_eq!(ts.current_played_notes(), exp);

    tc.shutdown().await
}
