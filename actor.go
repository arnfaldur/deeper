package main

import (
	"math"
	"math/cmplx"
	"math/rand"
)

type Entity struct {
	//Runtime computable members
	id  ID
	pos complex128
	vel complex128
	//Load-time members
	Name      string
	Collision bool
	Solid     bool
	Size      float64
	Weight    float64
}

type Character struct {
	Entity
	MaxHealth, CurrHealth int
	attributes            [7]int
	damage                int
}

type Player struct {
	Character
}

type NPC struct {
	Character
	Aggro bool
}

func NewNPC(name string, x float64, y float64) NPC {
	npc := metaCharacters[name]
	if npc.variations > 1 {
		npc.id.state = rand.Int() % npc.variations
	}
	npc.pos = complex(x, y)
	return npc.NPC
}

func (v *Character) attackBy(a Character) {
	v.CurrHealth -= a.damage
}

func getNPCsNear(pos complex128, radius float64) []*NPC {
	var npcs []*NPC
	for _, t := range vicinity(pos, radius) {
		if t[0] >= 0 && t[0] < MAPSIZE && t[1] >= 0 && t[1] < MAPSIZE {
			for i := range theMap[t[0]][t[1]].npcsOnTile {
				npcs = append(npcs, theMap[t[0]][t[1]].npcsOnTile[i])
			}
		}
	}
	return npcs
}

func (e *Entity) entityCollision() {

}

func (e *Entity) tileCollide(theMap *Mapt) {
	px, py := parts(e.pos + e.vel)

	pxf, pxr, pxc, pyf, pyr, pyc := int(px), int(px+0.5), int(math.Nextafter(px+1, math.Inf(-1))), int(py), int(py+0.5), int(math.Nextafter(py+1, math.Inf(-1)))
	any := false
	toWall := e.Size / 2
	if theMap[pyf][pxr].Collision && toWall >= math.Abs(py-0.5-float64(pyf)) {
		e.vel = complex(real(e.vel), math.Max(0, imag(e.vel)))
		//e.pos = complex(real(e.pos), float64(pyf)+0.5+toWall)
		any = true
	} else if theMap[pyc][pxr].Collision && toWall >= math.Abs(py+0.5-float64(pyc)) {
		e.vel = complex(real(e.vel), math.Min(0, imag(e.vel)))
		//e.pos = complex(real(e.pos), float64(pyc)-0.5-toWall)
		any = true
	}
	if theMap[pyr][pxf].Collision && toWall >= math.Abs(px-0.5-float64(pxf)) {
		e.vel = complex(math.Max(0, real(e.vel)), imag(e.vel))
		//e.pos = complex(float64(pxf)+0.5+toWall, imag(e.pos))
		any = true
	} else if theMap[pyr][pxc].Collision && toWall >= math.Abs(px+0.5-float64(pxc)) {
		e.vel = complex(math.Min(0, real(e.vel)), imag(e.vel))
		//e.pos = complex(float64(pxc)-0.5-toWall, imag(e.pos))
		any = true
	}
	if !any {
		fs := [4]float64{py - 0.5, py + 0.5, px - 0.5, px + 0.5}
		is := [4]int{pyf, pyc, pxf, pxc}
		for y := 0; y < 2; y++ {
			for x := 2; x < 4; x++ {
				colDir := complex(fs[x]-float64(is[x]), fs[y]-float64(is[y]))
				if theMap[is[y]][is[x]].Collision && toWall > cmplx.Abs(colDir) {
					colDep := toWall - cmplx.Abs(colDir)
					e.vel += cmplx.Rect(colDep/2, cmplx.Phase(colDir))
					//e.pos += cmplx.Rect(colDep, cmplx.Phase(colDir))
					break
				}
			}
		}
	}
}

func (c *Character) npcCollide(npcs *[]NPC) {
	netForce := 0 + 0i
	neighbours := getNPCsNear(c.pos, (c.Size+MAXCHARSIZE+sqrt2)/2)
	for i, n := range neighbours {
		colDir := (c.pos + c.vel) - (n.pos)
		colDep := (c.Size+n.Size)/2 - cmplx.Abs(colDir)
		if *c != n.Character && colDep > 0 {
			//nudge := cmul(colDir, colDep)
			nudge := cmplx.Rect(colDep/2, cmplx.Phase(colDir))
			//println(colDep)
			//c.pos += nudge
			//(*npcs)[i].pos -= nudge
			if c.id == hilbert.id {
				neighbours[i].CurrHealth -= c.damage
			} else {
				netForce += nudge
				//c.vel = c.vel + nudge
				//(*npcs)[i].vel = n.vel - nudge
			}
		}
	}
}

func (e *Entity) findCollisions(theMap Mapt) {
	for _, t := range vicinity(e.pos, e.Size+MAXCHARSIZE) {
		for _, n := range theMap[t[0]][t[1]].npcsOnTile {
			if cmplx.Abs(e.pos+e.vel-n.pos+n.vel) <= (e.Size+n.Size)/2 {

			}
		}
	}
}

func findAllCollisions(theMap Mapt, p Player, npcs []NPC) {
	p.findCollisions(theMap)
	for _, n := range npcs {
		n.findCollisions(theMap)
	}
}

func dealWithCollisions(theMap *Mapt, p *Player, npcs *[]NPC, moveDirection complex128) {
	if cmplx.Abs(moveDirection) > 1 {
		moveDirection = cmplxNorm(moveDirection)
	}
	p.vel = approach(p.vel, moveDirection/5) * complex(timeDilation, 0)
	for i, e := range *npcs {
		if e.Aggro {
			diff := p.pos - e.pos
			(*npcs)[i].vel = cmul(approach(e.vel, cmplxNorm(diff)/10+cmplx.Rect(0.01, (rand.Float64()*math.Pi*2)-math.Pi)), timeDilation)
		}
	}
	p.npcCollide(npcs)
	for i := range *npcs {
		(*npcs)[i].npcCollide(npcs)
	}
	p.tileCollide(theMap)
	for i := range *npcs {
		(*npcs)[i].tileCollide(theMap)
	}
	p.pos += p.vel
	for i := range *npcs {
		(*npcs)[i].pos += (*npcs)[i].vel
	}
}

func testEnemyNPC(pos complex128, id int) NPC {
	return NPC{Character{Entity: Entity{id: makeActorID(id), pos: pos, Name: "TestEnemy", Size: 0.8}, MaxHealth: 10, CurrHealth: 10}, true}
}

func dummyNPC(x, y int) NPC {
	return NPC{Character{Entity: Entity{id: DUMMY, pos: complex(float64(x), float64(y)), Name: "dummy"}, MaxHealth: 10, CurrHealth: 10}, true}
}

func approach(vel, target complex128) complex128 {
	return (vel*4 + target) / 5
}
