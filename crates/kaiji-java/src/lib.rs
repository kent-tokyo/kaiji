use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jfloat, jstring, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;

fn get_string(env: &mut JNIEnv, s: JString) -> Option<String> {
    env.get_string(&s).ok().map(|js| js.into())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_kenttokyo_kaiji_Kaiji_normalize<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    input: JString<'local>,
) -> jstring {
    let Some(s) = get_string(&mut env, input) else {
        return std::ptr::null_mut();
    };
    match ::kaiji::normalize_default(&s) {
        Ok(result) => env
            .new_string(result.as_ref())
            .map(|js| js.into_raw())
            .unwrap_or(std::ptr::null_mut()),
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_kenttokyo_kaiji_Kaiji_matches<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    a: JString<'local>,
    b: JString<'local>,
) -> jboolean {
    let (Some(sa), Some(sb)) = (get_string(&mut env, a), get_string(&mut env, b)) else {
        return JNI_FALSE;
    };
    match ::kaiji::matches_default(&sa, &sb) {
        Ok(true) => JNI_TRUE,
        _ => JNI_FALSE,
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_kenttokyo_kaiji_Kaiji_similarityScore<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    a: JString<'local>,
    b: JString<'local>,
) -> jfloat {
    let (Some(sa), Some(sb)) = (get_string(&mut env, a), get_string(&mut env, b)) else {
        return -1.0;
    };
    let cfg = ::kaiji::NormalizerConfig::default();
    ::kaiji::similarity_score(&sa, &sb, &cfg).unwrap_or(-1.0)
}
