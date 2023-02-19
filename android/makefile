start: debug
	@./gradlew -Dorg.gradle.java.home="${ANDROID_JDK_DIR}" app:installDebug
	@${ANDROID_HOME}/platform-tools/adb shell am start -n 'com.zhqli.wikit/.MainActivity'

debug:
	@./gradlew -Dorg.gradle.java.home="${ANDROID_JDK_DIR}" app:assembleDebug

release:
	@./gradlew -Dorg.gradle.java.home="${ANDROID_JDK_DIR}" app:assembleRelease
