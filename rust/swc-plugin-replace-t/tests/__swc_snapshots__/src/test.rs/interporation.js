export const object = ()=>{
    console.log(((v)=>`value: ${v.max} - ${v.min}`)({
        min: 100,
        max: 400
    }));
};
