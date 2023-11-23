import _TWSearchSwiftBridge
public func randomScrambleFor(_ event: Event) -> Optional<Alg> {
    { let val = __swift_bridge__$random_scramble_for_event_swift({event.isOwned = false; return event.ptr;}()); if val != nil { return Alg(ptr: val!) } else { return nil } }()
}

public class PuzzleError: PuzzleErrorRefMut {
    var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$PuzzleError$_free(ptr)
        }
    }
}
public class PuzzleErrorRefMut: PuzzleErrorRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class PuzzleErrorRef {
    var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension PuzzleError: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_PuzzleError$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_PuzzleError$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: PuzzleError) {
        __swift_bridge__$Vec_PuzzleError$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_PuzzleError$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (PuzzleError(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<PuzzleErrorRef> {
        let pointer = __swift_bridge__$Vec_PuzzleError$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return PuzzleErrorRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<PuzzleErrorRefMut> {
        let pointer = __swift_bridge__$Vec_PuzzleError$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return PuzzleErrorRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<PuzzleErrorRef> {
        UnsafePointer<PuzzleErrorRef>(OpaquePointer(__swift_bridge__$Vec_PuzzleError$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_PuzzleError$len(vecPtr)
    }
}


public class Alg: AlgRefMut {
    var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$Alg$_free(ptr)
        }
    }
}
public class AlgRefMut: AlgRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class AlgRef {
    var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension Alg: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_Alg$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_Alg$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: Alg) {
        __swift_bridge__$Vec_Alg$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_Alg$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (Alg(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<AlgRef> {
        let pointer = __swift_bridge__$Vec_Alg$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return AlgRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<AlgRefMut> {
        let pointer = __swift_bridge__$Vec_Alg$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return AlgRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<AlgRef> {
        UnsafePointer<AlgRef>(OpaquePointer(__swift_bridge__$Vec_Alg$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_Alg$len(vecPtr)
    }
}


public class Event: EventRefMut {
    var isOwned: Bool = true

    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }

    deinit {
        if isOwned {
            __swift_bridge__$Event$_free(ptr)
        }
    }
}
public class EventRefMut: EventRef {
    public override init(ptr: UnsafeMutableRawPointer) {
        super.init(ptr: ptr)
    }
}
public class EventRef {
    var ptr: UnsafeMutableRawPointer

    public init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }
}
extension Event: Vectorizable {
    public static func vecOfSelfNew() -> UnsafeMutableRawPointer {
        __swift_bridge__$Vec_Event$new()
    }

    public static func vecOfSelfFree(vecPtr: UnsafeMutableRawPointer) {
        __swift_bridge__$Vec_Event$drop(vecPtr)
    }

    public static func vecOfSelfPush(vecPtr: UnsafeMutableRawPointer, value: Event) {
        __swift_bridge__$Vec_Event$push(vecPtr, {value.isOwned = false; return value.ptr;}())
    }

    public static func vecOfSelfPop(vecPtr: UnsafeMutableRawPointer) -> Optional<Self> {
        let pointer = __swift_bridge__$Vec_Event$pop(vecPtr)
        if pointer == nil {
            return nil
        } else {
            return (Event(ptr: pointer!) as! Self)
        }
    }

    public static func vecOfSelfGet(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<EventRef> {
        let pointer = __swift_bridge__$Vec_Event$get(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return EventRef(ptr: pointer!)
        }
    }

    public static func vecOfSelfGetMut(vecPtr: UnsafeMutableRawPointer, index: UInt) -> Optional<EventRefMut> {
        let pointer = __swift_bridge__$Vec_Event$get_mut(vecPtr, index)
        if pointer == nil {
            return nil
        } else {
            return EventRefMut(ptr: pointer!)
        }
    }

    public static func vecOfSelfAsPtr(vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<EventRef> {
        UnsafePointer<EventRef>(OpaquePointer(__swift_bridge__$Vec_Event$as_ptr(vecPtr)))
    }

    public static func vecOfSelfLen(vecPtr: UnsafeMutableRawPointer) -> UInt {
        __swift_bridge__$Vec_Event$len(vecPtr)
    }
}



