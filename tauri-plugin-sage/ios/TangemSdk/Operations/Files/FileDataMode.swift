//
//  FileDataMode.swift
//  TangemSdk
//
//  Created by Andrew Son on 10/8/20.
//  Copyright © 2020 Tangem AG. All rights reserved.
//

import Foundation

enum FileDataMode: Byte, InteractionMode {
    
    /// Use this mode when reading file
    case read = 0
    
    /// Send this mode for creating file on card
    case initiateWritingFile = 1
    
    /// Send this mode when transfering file data on card
    case writeFile = 2
    
    /// Send this mode when saving fully transfered file on card
    case confirmWritingFile = 3
    
    /// Mode for deleting file command
    case deleteFile = 5
    
    /// Mode for updating file settings such as file visibility
    case changeFileSettings = 6
    
    /// Used for reading file hash for confirming that file not currupted
    static var readFileHash: FileDataMode {
        return .initiateWritingFile
    }
}
