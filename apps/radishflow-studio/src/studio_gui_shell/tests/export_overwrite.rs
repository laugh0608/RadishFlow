use super::*;
use equipment::CalculationTableFormat as Format;
use std::path::Path;

pub(super) fn check_target_changes(
    app: &mut ReadyAppState,
    path: &Path,
    select: impl Fn(&mut ReadyAppState, &Path),
    confirm: fn(&mut ReadyAppState),
    cancel: fn(&mut ReadyAppState),
) {
    select(app, path);
    let expected = fs::read(path).unwrap();
    let pending = |app: &ReadyAppState| {
        app.pending_result_export.is_some() || app.equipment.pending_export.is_some()
    };

    let assert_expired = |app: &ReadyAppState| {
        assert!(!pending(app));
        let notice = app.project_open.notice.as_ref().unwrap();
        assert_eq!(notice.level, ProjectOpenNoticeLevel::Warning);
        assert!(notice.detail.contains(&path.display().to_string()));
    };

    fs::write(path, b"original").unwrap();
    select(app, path);
    assert!(pending(app));
    fs::write(path, b"external").unwrap();
    confirm(app);
    assert_eq!(fs::read(path).unwrap(), b"external");
    assert_expired(app);

    select(app, path);
    assert!(pending(app));
    cancel(app);
    confirm(app);
    assert_eq!(fs::read(path).unwrap(), b"external");
    select(app, path);
    assert!(pending(app));
    confirm(app);
    assert!(fs::read(path).unwrap() == expected);

    select(app, path);
    assert!(pending(app));
    fs::remove_file(path).unwrap();
    confirm(app);
    assert!(!path.try_exists().unwrap());
    assert_expired(app);

    select(app, path);
    assert!(!pending(app));
    assert!(fs::read(path).unwrap() == expected);
    fs::remove_file(path).unwrap();
}

pub(super) fn check_equipment_formats(
    app: &mut ReadyAppState,
    path: &Path,
    select: impl Fn(&mut ReadyAppState, Format, &Path),
) {
    for format in [Format::Csv, Format::Json] {
        check_target_changes(
            app,
            &path.with_extension(format.extension()),
            |app, path| select(app, format, path),
            ReadyAppState::confirm_equipment_export,
            ReadyAppState::cancel_equipment_export,
        );
    }
}
