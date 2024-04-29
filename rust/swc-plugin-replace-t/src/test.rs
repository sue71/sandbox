use std::path::Path;

use swc_core::{
    common::{chain, Mark},
    ecma::{
        transforms::testing::{test, Tester},
        visit::{as_folder, Fold},
    },
};
use swc_ecma_parser::{Syntax, TsConfig};
use swc_ecma_transforms_react::react;

use crate::TransformVisitor;

fn syntax(tsx: bool) -> Syntax {
    Syntax::Typescript(TsConfig {
        tsx,
        ..Default::default()
    })
}

fn transformer(t: &Tester) -> impl Fold {
    let current_dir = std::env::current_dir().unwrap();
    let path = current_dir.join(Path::new("fixture/json"));
    let mark = Mark::new();
    chain!(
        react(
            t.cm.clone(),
            Some(t.comments.clone()),
            Default::default(),
            mark,
            mark
        ),
        as_folder(TransformVisitor::new(path.to_str().unwrap().to_string()))
    )
}

test!(
    syntax(false),
    |t| transformer(t),
    simple,
    r#"
    const noArgs = () => {
        const { t } = useTranslation('noArgs');

        console.log(t('a'));
        console.log(t('b'));
        console.log(t('x.y'));
        console.log(t('noArgs:a'));
        console.log(t('noArgs:x.y'));
    };

    function noArgs2() {
        const { t } = useTranslation('noArgs2');

        console.log(t('c'));
        console.log(t('d'));
    }
    "#
);

test!(
    syntax(false),
    |t| transformer(t),
    dir,
    r#"
    export const directory = () => {
        const { t } = useTranslation("dir/dir");
        console.log(t("foo.bar"));
    };

    export const directory2 = () => {
        const { t } = useTranslation(["dir/dir"]);
        console.log(t("dir/dir:foo.bar"));
    };
    "#
);

test!(
    syntax(false),
    |t| transformer(t),
    interporation,
    r#"
    export const object = () => {
        const { t } = useTranslation('object');
        console.log(
          t('foo', {
            min: 100,
            max: 400,
          })
        );
    };
    "#
);

test!(
    syntax(false),
    |t| transformer(t),
    remove_import_stmt,
    r#"
    import { foo } from 'other-module';
    import { useTranslation } from 'react-i18next';
    "#
);

test!(
    syntax(true),
    |t| transformer(t),
    component_interporation,
    r#"
    const ComponentWithTrans = () => {
        const { t } = useTranslation('react');
        const a = useMemo(() => t('foo'), [t]);
        const b = useCallback(() => {
          console.log(t('bar'));
        }, [t]);
        const c = 1;

        useEffect(() => {
          console.log(t('bar'));
        }, [t, c]);

        return (
          <div>
            <div>
              <Trans>{t('lineBreak')}</Trans>
            </div>
          </div>
        );
    };

    // https://github.com/i18next/react-i18next/blob/da07fbaed70310c464421e87688d9850647e7f2f/test/trans.render.spec.js
    const ComponentWithTransComponents = () => {
        const { t } = useTranslation('react');

        return (
          <Trans components={{ link: (<Link href="foo.com" />), button: <Button /> }}>
            {t('transComponents')}
          </Trans>
        );
    };
    "#
);
