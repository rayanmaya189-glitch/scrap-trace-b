# B-Trace Mobile App - Android (Kotlin + Jetpack Compose)

## Project Structure
```
mobile/android/
├── app/
│   ├── src/main/
│   │   ├── java/com/btrace/app/
│   │   │   ├── MainActivity.kt
│   │   │   ├── BTraceApplication.kt
│   │   │   ├── ui/
│   │   │   ├── data/
│   │   │   ├── domain/
│   │   │   └── di/
│   │   ├── res/
│   │   └── AndroidManifest.xml
│   └── build.gradle.kts
├── gradle/
├── build.gradle.kts
└── settings.gradle.kts
```

## Tech Stack
- **Language**: Kotlin 1.9+
- **UI Framework**: Jetpack Compose
- **Architecture**: MVVM + Clean Architecture
- **Dependency Injection**: Hilt
- **Networking**: Retrofit + OkHttp
- **Local Database**: Room
- **Async**: Coroutines + Flow
- **Navigation**: Compose Navigation
- **State Management**: ViewModel + StateFlow
