# B-Trace Mobile App - iOS (Swift + SwiftUI)

## Project Structure
```
mobile/ios/BTraceApp/
├── BTraceApp/
│   ├── BTraceApp.swift
│   ├── AppDelegate.swift
│   ├── Info.plist
│   ├── Models/
│   ├── Services/
│   ├── ViewModels/
│   ├── Views/
│   │   ├── Auth/
│   │   ├── Dashboard/
│   │   ├── Materials/
│   │   ├── Handshakes/
│   │   ├── Scores/
│   │   ├── Compliance/
│   │   └── Profile/
│   ├── Components/
│   ├── Utilities/
│   └── Resources/
├── BTraceApp.xcodeproj
└── Podfile
```

## Tech Stack
- **Language**: Swift 5.9+
- **UI Framework**: SwiftUI
- **Architecture**: MVVM + Clean Architecture
- **Dependency Injection**: Manual DI / Swinject
- **Networking**: URLSession + Codable
- **Local Database**: CoreData / Realm
- **Async**: async/await + Combine
- **State Management**: @StateObject, @ObservedObject

## Minimum Requirements
- iOS 16.0+
- Xcode 15.0+
- Swift 5.9+
