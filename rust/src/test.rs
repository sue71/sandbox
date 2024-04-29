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
        as_folder(TransformVisitor::new(path.to_str().unwrap().to_string())),
        react(
            t.cm.clone(),
            Some(t.comments.clone()),
            Default::default(),
            mark
        )
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
    "#,
    r#"
    const noArgs = () => {

        console.log("test1");
        console.log("line1\nline2");
        console.log("test2");
        console.log("test1");
        console.log("test2");
    };

    function noArgs2() {

        console.log("test3");
        console.log("test4");
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
    "#,
    r#"
    export const directory = () => {
        console.log("hello");
    };

    export const directory2 = () => {
        console.log("hello");
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
    "#,
    r#"
    export const object = () => {
        console.log((v => `value: ${v.max} - ${v.min}`)({
            min: 100,
            max: 400
        }));
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
    "#,
    r#"
    import { foo } from 'other-module';
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
    "#,
    r#"
    const ComponentWithTrans = ()=>{
        const a = useMemo(()=>"hi", []);
        const b = useCallback(()=>{
            console.log("hi!");
        }, []);
        const c = 1;
        useEffect(()=>{
            console.log("hi!");
        }, [
            c
        ]);
        return React.createElement("div", null, React.createElement("div", null, React.createElement(React.Fragment, null, React.createElement(React.Fragment, null, "hello", React.createElement("br", null), "world", React.createElement("br", null), "hi!"))));
    };
    const ComponentWithTransComponents = ()=>{
        return React.createElement(React.Fragment, null, React.createElement(React.Fragment, null, React.createElement(Link, {
            href: "foo.com"
        }, "aaaaa"), React.createElement(Button, null)));
    };
    "#
);
