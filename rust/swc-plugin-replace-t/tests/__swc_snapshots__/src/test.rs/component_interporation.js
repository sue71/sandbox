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
    return /*#__PURE__*/ React.createElement("div", null, /*#__PURE__*/ React.createElement("div", null, /*#__PURE__*/ React.createElement(Trans, null, <>hello<br/>world<br/>hi!</>)));
};
// https://github.com/i18next/react-i18next/blob/da07fbaed70310c464421e87688d9850647e7f2f/test/trans.render.spec.js
const ComponentWithTransComponents = ()=>{
    return /*#__PURE__*/ React.createElement(Trans, {
        components: {
            link: /*#__PURE__*/ React.createElement(Link, {
                href: "foo.com"
            }),
            button: /*#__PURE__*/ React.createElement(Button, null)
        }
    }, <><link>aaaaa</link><button/></>);
};
